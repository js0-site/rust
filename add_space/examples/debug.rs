use add_space::add_space;

fn main() {
  println!("1: {:?}", add_space("./dev.js ./ui/webc.site/组件名"));
  println!("2: {:?}", add_space("`./dev.js ./ui/webc.site/组件名`"));
  println!("3: {:?}", add_space("`./dev.js ./ui/webc.site/组件名`保存"));
  println!("4: {:?}", add_space("现在  `./dev.js ./ui/webc.site/组件名` 保存会变成  `./dev.js ./ui/webc.site/组件名 ` ，`前面多了一个空格"));
}
