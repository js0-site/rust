use aok::{OK, Void};
use citer::CIter;

#[static_init::constructor(0)]
extern "C" fn _log_init() {
  log_init::init();
}

#[test]
fn test_citer_basic() -> Void {
  // 测试基本循环迭代器功能 / Test basic circular iterator functionality
  let data = [1, 2, 3, 4, 5];
  let mut iter = CIter::new(&data, 0);

  // 验证前5个元素 / Verify first 5 elements
  assert_eq!(iter.next(), Some((0, &1)));
  assert_eq!(iter.next(), Some((1, &2)));
  assert_eq!(iter.next(), Some((2, &3)));
  assert_eq!(iter.next(), Some((3, &4)));
  assert_eq!(iter.next(), Some((4, &5)));

  // 迭代器应该结束 / Iterator should end
  assert_eq!(iter.next(), None);

  OK
}

#[test]
fn test_citer_position() -> Void {
  // 测试位置功能 / Test position functionality
  let data = [10, 20, 30];
  let iter = CIter::new(&data, 2);

  // 位置应该是1 (idx-1) / Position should be 1 (idx-1)
  assert_eq!(iter.pos(), 1);

  let iter_zero = CIter::new(&data, 0);
  // 位置0时应该返回0 / Position should be 0 when idx is 0
  assert_eq!(iter_zero.pos(), 0);

  OK
}

#[test]
fn test_citer_from_middle() -> Void {
  // 测试从中间位置开始迭代 / Test iteration starting from middle position
  let data = [1, 2, 3, 4];
  let mut iter = CIter::new(&data, 2);

  // 从索引2开始，应该是3, 4, 1, 2 / Starting from index 2, should be 3, 4, 1, 2
  assert_eq!(iter.next(), Some((2, &3)));
  assert_eq!(iter.next(), Some((3, &4)));
  assert_eq!(iter.next(), Some((0, &1)));
  assert_eq!(iter.next(), Some((1, &2)));
  assert_eq!(iter.next(), None);

  OK
}

#[test]
fn test_citer_empty_slice() -> Void {
  // 测试空切片 / Test empty slice
  let data: [i32; 0] = [];
  let mut iter = CIter::new(&data, 0);

  // 空切片应该立即返回None / Empty slice should immediately return None
  assert_eq!(iter.next(), None);

  OK
}

#[test]
fn test_citer_single_element() -> Void {
  // 测试单元素切片 / Test single element slice
  let data = [42];
  let mut iter = CIter::new(&data, 0);

  // 应该返回唯一元素然后结束 / Should return the single element then end
  assert_eq!(iter.next(), Some((0, &42)));
  assert_eq!(iter.next(), None);

  OK
}

#[test]
fn test_citer_out_of_bounds_start() -> Void {
  // 测试超出边界的起始位置 / Test out of bounds start position
  let data = [1, 2, 3];
  let mut iter = CIter::new(&data, 5); // 5 % 3 = 2

  // 应该从索引2开始 / Should start from index 2
  assert_eq!(iter.next(), Some((2, &3)));
  assert_eq!(iter.next(), Some((0, &1)));
  assert_eq!(iter.next(), Some((1, &2)));
  assert_eq!(iter.next(), None);

  OK
}

#[cfg(feature = "rand")]
#[test]
fn test_citer_rand() -> Void {
  // 测试随机起始位置 / Test random start position
  let data = [1, 2, 3, 4, 5];
  let iter = CIter::rand(&data);

  // 验证迭代器被正确创建 / Verify iterator is properly created
  assert_eq!(iter.li.len(), 5);

  // 测试随机迭代器能正常工作 / Test that random iterator works properly
  let collected: Vec<&i32> = iter.map(|(_, value)| value).collect();
  assert_eq!(collected.len(), 5);

  OK
}

#[test]
fn test_citer_collect() -> Void {
  // 测试收集所有元素 / Test collecting all elements
  let data = [1, 2, 3];
  let iter = CIter::new(&data, 1);
  let collected: Vec<(usize, &i32)> = iter.collect();

  // 应该收集到所有3个元素 / Should collect all 3 elements
  assert_eq!(collected.len(), 3);
  assert_eq!(collected, vec![(1, &2), (2, &3), (0, &1)]);

  OK
}
