用到的模块、函数都尽量在文件开头导入
use 要写明具体的导入模块, 禁止用 use *
错误都用 thiserror 在 src/error.rs 中定义，并定义 Result，在 lib.rs 中 `use error::{Result, Error};`，避免使用字符串作为错误
用 as 等数字转换要小心静默失败，但是要是 100%确定没问题，就不用 try_into，大胆用 as 提高性能
禁止使用 panic
用 as 向下转换数字要小心静默失败
约束要最小化（避免冗余的约束）
参数要秉承最小化原则,能传部分属性,就不要传入整个结构体
避免不必要的复制 clone 开销，时间复杂度、空间复杂度都要追求完美
不写重复的代码，不写雷同的类，可通过函数抽象+泛型参数、泛型 trait 等编程技巧减少冗余代码
函数名,变量名都要简洁，不要起冗长的名
避免雷同的字符串，请定义为常量
安装包依赖都用 cargo add 和 cargo add -F,不可直接编辑 Cargo.toml
Cargo.toml 的 edition 用 2024, rust 要用最新写法，禁止用过时的模块
注释都写英文和中文双语,注释要简洁，不必注释显而易见的东西,不要写语言名做前缀
格式化字符串尽量把变量名写到字符串中,比如 format!("{varname}")
遇到问题,多用 dbg!进行调试
追求性能的极致，用 fastrand、hipstr(HipByt、HipStr)、parking_lot、coarsetime 等高性能库替换标准库
如果需要一个并发读写的字典, 用 papaya
./src/ 中一级模块的公开函数和结构体都在 ./src/lib.rs 导出，模块间相互引用函数用 crate::函数名， 而不是 crate::模块名::函数名
日志用 log；测试中用下面代码初始化日志显示
```
#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}
```
每次写完记得 ./test.sh 测试编译