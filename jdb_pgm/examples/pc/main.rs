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
  let n = 1_000_000;
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
