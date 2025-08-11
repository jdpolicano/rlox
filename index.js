class MyClass {
  name() {
    return this.name;
  }
}

mc = new MyClass();
mc.name = "jacob";

console.log(mc.name());
