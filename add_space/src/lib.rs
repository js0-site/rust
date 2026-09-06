pub mod error;

pub use error::{Error, Result};
use std::{iter::Peekable, str::Chars};

use unicode_script::{Script, UnicodeScript};

/// Character state enum.
/// 字符状态枚举
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum State {
  Space,
  Char,
  Letter,
  Punctuation,
}

/// Determine the state of a character.
/// 判断字符的状态
#[inline]
pub fn state(c: char) -> State {
  // ASCII fast path to avoid unicode-script lookup overhead.
  // ASCII 快速通道，避免 unicode-script 查表开销
  if c.is_ascii() {
    if c.is_ascii_whitespace() {
      return State::Space;
    }
    if c == '`' {
      return State::Letter;
    }
    if matches!(
      c,
      '!'
        | '"'
        | '#'
        | '%'
        | '\\'
        | '\''
        | '*'
        | '+'
        | ','
        | '-'
        | '.'
        | '/'
        | ':'
        | '<'
        | '='
        | '>'
        | '?'
        | '@'
        | '^'
    ) {
      return State::Punctuation;
    }
    return State::Letter;
  }

  // Non-ASCII character handling.
  // 非 ASCII 字符处理
  if matches!(
    c.script(),
    Script::Han
      | Script::Hiragana
      | Script::Katakana
      | Script::Thai
      | Script::Lao
      | Script::Khmer
      | Script::Myanmar
      | Script::Tibetan
  ) || ('０'..='９').contains(&c)
  {
    return State::Char;
  }

  if matches!(c, '　') {
    return State::Space;
  }

  if matches!(
    c,
    '·'
      | '—'
      | '‘'
      | '’'
      | '“'
      | '”'
      | '…'
      | '、'
      | '。'
      | '「'
      | '」'
      | '『'
      | '』'
      | '！'
      | '，'
      | '：'
      | '？'
      | '；'
      | '（'
      | '）'
      | '《'
      | '》'
      | '〈'
      | '〉'
      | '【'
      | '】'
      | '〔'
      | '〕'
      | '〖'
      | '〗'
      | '〘'
      | '〙'
      | '〚'
      | '〛'
      | '〜'
      | '～'
      | '［'
      | '］'
      | '｛'
      | '｝'
  ) || (c.len_utf8() > 1 && unic_emoji_char::is_emoji(c))
  {
    return State::Punctuation;
  }

  State::Letter
}

/// Push opening brackets to stack.
/// 将左括号入栈
#[inline]
fn push_stack(c: char, stack: &mut Vec<char>) {
  if matches!(c, '[' | '(' | '{') {
    stack.push(c);
  }
}

/// Check if the state is letter or punctuation.
/// 判断状态是否为字母或标点
#[inline]
pub fn state_is_letter_or_punctuation(s: State) -> bool {
  matches!(s, State::Letter | State::Punctuation)
}

/// Check if character is Chinese year, month, or day.
/// 判断字符是否为中文年月日
#[inline]
fn is_ymd(c: char) -> bool {
  matches!(c, '年' | '月' | '日')
}

/// Trim trailing spaces between digits and CJK date units (年月日).
/// 修剪数字与中文年月日之间的尾随空格
#[inline]
fn trim_date_space(c: char, r: &mut String, pre_c: &mut char, pre_state: &mut State) {
  let is_digit_c = c.is_ascii_digit();
  let is_ymd_c = is_ymd(c);

  if !is_digit_c && !is_ymd_c {
    return;
  }

  if let Some(ch) = r.chars().rev().find(|&ch| ch != ' ' && ch != '\t')
    && ((is_ymd_c && ch.is_ascii_digit()) || (is_digit_c && is_ymd(ch)))
  {
    let len = r.trim_end_matches([' ', '\t']).len();
    r.truncate(len);
    if let Some(last_c) = r.chars().next_back() {
      *pre_c = last_c;
      *pre_state = state(last_c);
    }
  }
}

/// Check if there is a matching unescaped quote character ahead.
/// 检查前方是否存在匹配的且未转义的引号字符
fn has_matching_quote(iter: Peekable<Chars<'_>>, quote: char) -> bool {
  let mut escaped = false;
  for c in iter {
    if escaped {
      escaped = false;
    } else if c == '\\' {
      escaped = true;
    } else if c == quote {
      return true;
    }
  }
  false
}

/// Automatically add spaces between Chinese characters and English letters/digits.
/// 自动在中文和英文/数字之间添加空格
pub fn add_space(txt: impl AsRef<str>) -> String {
  let txt = txt.as_ref();
  let mut r = String::with_capacity(txt.len());
  let mut iter = txt.chars().peekable();

  if let Some(c) = iter.next() {
    r.push(c);
    let mut is_escape = c == '\\';
    let mut in_quote =
      if !is_escape && (c == '"' || c == '\'') && has_matching_quote(iter.clone(), c) {
        Some(c)
      } else {
        None
      };

    // Track backtick state and length, Some(n) means matching n backticks.
    // 追踪反引号状态及长度，Some(n) 表示匹配 n 个反引号
    let mut backtick_style = if c == '`' {
      let mut count = 1;
      while iter.peek() == Some(&'`') {
        iter.next();
        count += 1;
      }
      r.extend(std::iter::repeat_n('`', count - 1));
      Some(count)
    } else {
      None
    };
    let mut pre_state = state(c);
    let mut pre_pre_state = State::Space;
    let mut pre_c = c;
    let mut stack = Vec::new();

    push_stack(c, &mut stack);

    // Helper closure: determine if spaces are needed at the boundaries of backtick blocks.
    // 辅助闭包：判断是否需要在反引号块边界加空格
    let should_space = |pre_state: State, next_c: Option<&char>| -> bool {
      next_c.is_some_and(|&nc| {
        let next_s = state(nc);
        (pre_state == State::Letter && next_s == State::Char)
          || (pre_state == State::Char && next_s == State::Letter)
      })
    };

    #[allow(clippy::while_let_on_iterator)]
    while let Some(c) = iter.next() {
      if is_escape {
        is_escape = false;
        r.push(c);
        continue;
      }

      if c == '`' {
        let mut count = 1;
        while iter.peek() == Some(&'`') {
          iter.next();
          count += 1;
        }

        match backtick_style {
          None => {
            backtick_style = Some(count);
            if should_space(pre_state, iter.peek()) {
              r.push(' ');
            }
            r.extend(std::iter::repeat_n('`', count));
            pre_pre_state = pre_state;
            pre_state = State::Letter;
            pre_c = '`';
          }
          Some(style_len) => {
            if count == style_len {
              backtick_style = None;
              let space_needed = should_space(pre_state, iter.peek());
              r.extend(std::iter::repeat_n('`', count));
              if space_needed {
                r.push(' ');
                pre_pre_state = pre_state;
                pre_state = State::Space;
                pre_c = ' ';
              } else {
                pre_pre_state = pre_state;
                pre_state = State::Letter;
                pre_c = '`';
              }
            } else {
              r.extend(std::iter::repeat_n('`', count));
              pre_pre_state = pre_state;
              pre_state = State::Letter;
              pre_c = '`';
            }
          }
        }
        continue;
      }

      if backtick_style.is_some() {
        r.push(c);
        if c == '\\' {
          is_escape = true;
        }
        pre_pre_state = pre_state;
        pre_state = state(c);
        pre_c = c;
        continue;
      }

      let s = state(c);

      let mut current_in_quote = in_quote;
      if in_quote.is_none() {
        if (c == '"' || c == '\'') && has_matching_quote(iter.clone(), c) {
          in_quote = Some(c);
          current_in_quote = Some(c);
        }
      } else if Some(c) == in_quote {
        in_quote = None;
      }

      if c == '\\' {
        is_escape = true;
      }

      if current_in_quote.is_some() {
        r.push(c);
      } else {
        trim_date_space(c, &mut r, &mut pre_c, &mut pre_state);
        push_stack(c, &mut stack);
        match s {
          State::Char => {
            if pre_state == State::Letter
              && !matches!(pre_c, '[' | '(' | '{')
              && state_is_letter_or_punctuation(pre_pre_state)
              && !(is_ymd(c) && pre_c.is_ascii_digit())
            {
              r.push(' ');
            }
            r.push(c);
          }
          State::Letter => {
            if let Some(stack_last) = stack.last() {
              if matches!((stack_last, c), ('[', ']') | ('(', ')') | ('{', '}')) {
                stack.pop();
              }
            } else if ((pre_state == State::Char)
              || (matches!(pre_c, ',' | '…'))
              || (matches!(pre_c, '!' | '?')
                && !matches!(pre_pre_state, State::Letter | State::Punctuation)))
              && let Some(nc) = iter.peek()
              && state_is_letter_or_punctuation(state(*nc))
              && !(c.is_ascii_digit() && is_ymd(pre_c))
            {
              r.push(' ');
            }
            r.push(c);
          }
          _ => r.push(c),
        }
      }
      pre_pre_state = pre_state;
      pre_state = s;
      pre_c = c;
    }
  }
  r
}
