mod lang;
mod library;
mod pgm;
mod report;
mod scenario;
mod sucds;
mod vec;

use lang::detect_lang;
use library::Library;
use pgm::PgmLib;
use report::run;
use scenario::{DocIds, KeyOffsets, Scenario};
use sucds::SucdsLib;
use vec::VecLib;

fn main() {
  // 1000 MiB = 1000 * 1024 * 1024 bytes / 8 bytes/u64 = 131,072,000 elements
  let n = 131_072_000;
  let lang = detect_lang();

  // KeyOffsets
  let d = KeyOffsets::generate(n);
  let libs = [
    (PgmLib::NAME, PgmLib::measure(&d)),
    (SucdsLib::NAME, SucdsLib::measure(&d)),
    (VecLib::NAME, VecLib::measure(&d)),
  ];
  run::<KeyOffsets>(lang.as_ref(), &libs, n);

  // DocIds
  let d = DocIds::generate(n);
  let libs = [
    (PgmLib::NAME, PgmLib::measure(&d)),
    (SucdsLib::NAME, SucdsLib::measure(&d)),
    (VecLib::NAME, VecLib::measure(&d)),
  ];
  run::<DocIds>(lang.as_ref(), &libs, n);
}
