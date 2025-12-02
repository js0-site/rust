#!/usr/bin/env node

import net from 'net';
import tls from 'tls';
import { createHash } from 'crypto';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

// 创建一个模拟服务器间邮件投递的函数
class SMTPClient {
  constructor(host, port, useTLS = false) {
    this.host = host;
    this.port = port;
    this.useTLS = useTLS;
    this.socket = null;
    this.isSecure = false;
  }

  async connect() {
    return new Promise((resolve, reject) => {
      console.log(`正在连接到SMTP服务器 ${this.host}:${this.port}...`);
      
      if (this.useTLS) {
        // 对于端口465，直接使用TLS连接
        this.socket = tls.connect({
          host: this.host,
          port: this.port,
          rejectUnauthorized: false
        });
        this.isSecure = true;
      } else {
        this.socket = new net.Socket();
        this.socket.connect(this.port, this.host);
      }

      this.socket.setTimeout(30000); // 30秒超时

      this.socket.on('connect', async () => {
        console.log('✅ 连接成功');
        
        if (!this.useTLS) {
          // 对于端口587，需要升级到TLS
          if (this.port === 587) {
            await this.upgradeToTLS();
          }
        }
        
        // 读取服务器欢迎消息
        this.readResponse((response) => {
          console.log(`服务器响应: ${response}`);
          resolve();
        });
      });

      this.socket.on('error', (err) => {
        console.error(`❌ 连接错误: ${err.message}`);
        reject(err);
      });

      this.socket.on('timeout', () => {
        console.error('❌ 连接超时');
        this.socket.destroy();
        reject(new Error('Connection timeout'));
      });
    });
  }

  async upgradeToTLS() {
    return new Promise((resolve, reject) => {
      console.log('正在升级到TLS连接...');
      this.sendCommand('STARTTLS', (response) => {
        if (response.startsWith('220')) {
          // 升级到安全连接
          const secureSocket = tls.connect({
            socket: this.socket,
            servername: this.host,
            rejectUnauthorized: false
          });

          secureSocket.on('secureConnect', () => {
            console.log('✅ TLS连接已建立');
            this.socket = secureSocket;
            this.isSecure = true;
            resolve();
          });

          secureSocket.on('error', (err) => {
            console.error(`❌ TLS升级失败: ${err.message}`);
            reject(err);
          });
        } else {
          console.error(`❌ STARTTLS命令失败: ${response}`);
          reject(new Error(`STARTTLS failed: ${response}`));
        }
      });
    });
  }

  readResponse(callback) {
    let response = '';
    
    const dataHandler = (data) => {
      response += data.toString();
      
      // 检查是否是完整的SMTP响应（以\r\n结尾且包含完整的响应码）
      if (response.includes('\r\n')) {
        // SMTP响应通常以三位数字开头，后跟空格或减号，然后是消息
        const lines = response.split('\r\n');
        // 检查最后一条完整消息是否以三位数字响应码结尾
        for (let i = 0; i < lines.length - 1; i++) {
          const line = lines[i];
          if (line && /^\d{3}/.test(line)) {
            // 完整响应，移除监听器并回调
            this.socket.removeListener('data', dataHandler);
            callback(line);
            return;
          }
        }
      }
    };
    
    this.socket.on('data', dataHandler);
  }

  sendCommand(command, callback) {
    console.log(`发送命令: ${command}`);
    this.socket.write(command + '\r\n');
    
    this.readResponse(callback);
  }

  async authenticate(username, password) {
    return new Promise((resolve, reject) => {
      // 发送EHLO命令
      this.sendCommand(`EHLO ${this.host}`, (response) => {
        console.log(`EHLO响应: ${response}`);
        
        // 尝试登录认证
        this.sendCommand('AUTH LOGIN', (authResponse) => {
          if (authResponse.startsWith('334')) {
            // 服务器要求用户名
            const encodedUsername = Buffer.from(username).toString('base64');
            this.sendCommand(encodedUsername, (usernameResponse) => {
              if (usernameResponse.startsWith('334')) {
                // 服务器要求密码
                const encodedPassword = Buffer.from(password).toString('base64');
                this.sendCommand(encodedPassword, (passwordResponse) => {
                  if (passwordResponse.startsWith('235')) {
                    console.log('✅ 认证成功');
                    resolve();
                  } else {
                    console.error(`❌ 认证失败: ${passwordResponse}`);
                    reject(new Error(`Authentication failed: ${passwordResponse}`));
                  }
                });
              } else {
                console.error(`❌ 用户名认证失败: ${usernameResponse}`);
                reject(new Error(`Username auth failed: ${usernameResponse}`));
              }
            });
          } else {
            console.error(`❌ 认证请求失败: ${authResponse}`);
            reject(new Error(`AUTH LOGIN failed: ${authResponse}`));
          }
        });
      });
    });
  }

  async sendMail(from, to, subject, body) {
    return new Promise((resolve, reject) => {
      // 发送MAIL FROM命令
      this.sendCommand(`MAIL FROM:<${from}>`, (fromResponse) => {
        console.log(`MAIL FROM响应: ${fromResponse}`);
        
        if (!fromResponse.startsWith('250')) {
          reject(new Error(`MAIL FROM failed: ${fromResponse}`));
          return;
        }
        
        // 发送RCPT TO命令
        this.sendCommand(`RCPT TO:<${to}>`, (toResponse) => {
          console.log(`RCPT TO响应: ${toResponse}`);
          
          if (!toResponse.startsWith('250')) {
            reject(new Error(`RCPT TO failed: ${toResponse}`));
            return;
          }
          
          // 发送DATA命令
          this.sendCommand('DATA', (dataResponse) => {
            console.log(`DATA响应: ${dataResponse}`);
            
            if (!dataResponse.startsWith('354')) {
              reject(new Error(`DATA command failed: ${dataResponse}`));
              return;
            }
            
            // 构建邮件内容
            const emailContent = [
              `From: ${from}`,
              `To: ${to}`,
              `Subject: ${subject}`,
              'Content-Type: text/plain; charset=utf-8',
              'MIME-Version: 1.0',
              '', // 空行分隔头部和正文
              body,
              '.', // 结束符
            ].join('\r\n');
            
            console.log('正在发送邮件内容...');
            this.socket.write(emailContent + '\r\n');
            
            // 读取最终响应
            this.readResponse((finalResponse) => {
              console.log(`邮件发送响应: ${finalResponse}`);
              
              if (finalResponse.startsWith('250')) {
                console.log('✅ 邮件发送成功');
                resolve(finalResponse);
              } else {
                reject(new Error(`Email send failed: ${finalResponse}`));
              }
            });
          });
        });
      });
    });
  }

  async disconnect() {
    if (this.socket) {
      this.sendCommand('QUIT', (response) => {
        console.log(`服务器断开响应: ${response}`);
      });
      this.socket.destroy();
    }
  }
}

// 测试邮件发送
async function testSMTPDelivery() {
  // 注意：实际使用时需要替换为真实的SMTP服务器信息
  const smtpHost = 'smtp.qq.com';
  const smtpPort = 465; // 使用加密端口465
  const useTLS = true;
  
  // 注意：以下凭据仅为示例，请使用真实的邮箱凭据
  const email = process.env.SMTP_EMAIL || 'your_email@qq.com';
  const password = process.env.SMTP_PASSWORD || 'your_password';
  
  // 如果没有环境变量，则提示用户
  if (!process.env.SMTP_EMAIL) {
    console.log('💡 提示: 请设置环境变量 SMTP_EMAIL 和 SMTP_PASSWORD');
    console.log('例如: export SMTP_EMAIL="your_email@qq.com"');
    console.log('      export SMTP_PASSWORD="your_app_password"');
    console.log('对于QQ邮箱，密码应为邮箱授权码');
    return;
  }

  const client = new SMTPClient(smtpHost, smtpPort, useTLS);

  try {
    // 连接到SMTP服务器
    await client.connect();
    
    // 认证
    await client.authenticate(email, password);
    
    // 发送测试邮件
    const from = email;
    const to = email; // 发送到自己进行测试
    const subject = '服务器间邮件投递测试';
    const body = '这是一封通过SMTP协议直接投递的测试邮件，模拟服务器之间的邮件传输。';
    
    await client.sendMail(from, to, subject, body);
    
    console.log('\n🎉 邮件发送流程完成！');
  } catch (error) {
    console.error(`\n❌ 邮件发送失败: ${error.message}`);
  } finally {
    // 断开连接
    await client.disconnect();
  }
}

// 如果直接运行此文件，则执行测试
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// 使用一种方式来检测是否是直接运行此模块
if (process.argv[1] === __filename) {
  console.log('=== 开始服务器间邮件投递测试 ===\n');
  testSMTPDelivery().catch(console.error);
}

export default SMTPClient;
