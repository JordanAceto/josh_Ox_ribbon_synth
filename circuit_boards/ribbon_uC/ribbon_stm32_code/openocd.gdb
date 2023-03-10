file target/thumbv7em-none-eabihf/release/l412_test
target remote :3333

set print asm-demangle on
set print pretty on

load

break main

continue