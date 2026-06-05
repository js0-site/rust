use unicode_script::{Script, UnicodeScript};

pub fn state(c: char) -> State {
  if c.is_whitespace() {
    return State::Space;
  }
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

  if c == '`' {
    return State::Letter;
  }

  if matches!(
    c,
    '!' | '"'
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
      | '·'
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
  ) || (c.len_utf8() > 1 && unic_emoji_char::is_emoji(c))
  {
    return State::Punctuation;
  }

  State::Letter
}

fn push_stack(c: char, stack: &mut Vec<char>) {
  if "[({".contains(c) {
    stack.push(c);
  }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum State {
  Space,
  Char,
  Letter,
  Punctuation,
}

pub fn state_is_letter_or_punctuation(s: State) -> bool {
  matches!(s, State::Letter | State::Punctuation)
}

// Check if there is a matching unescaped quote character ahead.
// 检查前方是否存在匹配的且未转义的引号字符。
fn has_matching_quote(mut iter: std::iter::Peekable<std::str::Chars<'_>>, quote: char) -> bool {
  let mut escaped = false;
  while let Some(c) = iter.next() {
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

pub fn add_space(txt: impl AsRef<str>) -> String {
  let txt = txt.as_ref();
  let mut r = String::new();
  let mut iter = txt.chars().peekable();

  if let Some(c) = iter.next() {
    r.push(c);
    let mut is_escape = c == '\\';
    let mut in_quote = if !is_escape && (c == '"' || c == '\'') && has_matching_quote(iter.clone(), c) {
      Some(c)
    } else {
      None
    };
    let mut in_backtick = c == '`';
    let mut pre_state = state(c);
    let mut pre_pre_state = State::Space;
    let mut pre_c = c;
    let mut stack = Vec::new();

    push_stack(c, &mut stack);

    while let Some(c) = iter.next() {
      if is_escape {
        is_escape = false;
        r.push(c);
        continue;
      }

      if c == '`' {
        in_backtick = !in_backtick;
        let should_space = iter.peek().map_or(false, |next_c| {
          let next_s = state(*next_c);
          (pre_state == State::Letter && next_s == State::Char)
            || (pre_state == State::Char && next_s == State::Letter)
        });

        if in_backtick {
          if should_space {
            r.push(' ');
          }
        } else if should_space {
          r.push(c);
          r.push(' ');
          pre_pre_state = pre_state;
          pre_state = State::Space;
          pre_c = ' ';
          continue;
        }
        r.push(c);
        if c == '\\' {
          is_escape = true;
        }
        pre_pre_state = pre_state;
        pre_state = state(c);
        pre_c = c;
        continue;
      }

      if in_backtick {
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
        push_stack(c, &mut stack);
        match s {
          State::Char => {
            if pre_state == State::Letter
              && !"[({".contains(pre_c)
              && state_is_letter_or_punctuation(pre_pre_state)
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
              || (",…".contains(pre_c))
              || ("!?".contains(pre_c)
                && !matches!(pre_pre_state, State::Letter | State::Punctuation))
              || (pre_c == '.' && c.is_uppercase()))
              && let Some(c) = iter.peek()
              && state_is_letter_or_punctuation(state(*c))
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
