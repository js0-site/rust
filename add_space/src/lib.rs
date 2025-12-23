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

  if r##"!"#%\'*+,-.:<=>?@^·—‘’“”…、。「」『』！，：？；（）"##.contains(c)
    || (c.len_utf8() > 1 && unic_emoji_char::is_emoji(c))
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

pub fn add_space(txt: impl AsRef<str>) -> String {
  let txt = txt.as_ref();
  let mut r = String::new();
  let mut iter = txt.chars().peekable();

  if let Some(c) = iter.next() {
    r.push(c);
    let mut is_escape = c == '\\';
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
      let s = state(c);
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
          is_escape = c == '\\';
          if let Some(stack_last) = stack.last() {
            if matches!((stack_last, c), ('[', ']') | ('(', ')') | ('{', '}')) {
              stack.pop();
            }
          } else if ((!is_escape && pre_state == State::Char)
            || (",?!…".contains(pre_c))
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
      pre_pre_state = pre_state;
      pre_state = s;
      pre_c = c;
    }
  }
  r
}
