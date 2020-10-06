Build with:

```
cargo build --example=bootrom -Zbuild-std=core --release --target=examples/bootrom/x86_64-none.json
```

Strip
```
sstrip ./target/x86_64-none/release/examples/bootrom
```

Run
```
qemu-system-x86_64 \                  
-machine type=q35,accel=kvm \
-cpu host,-vmx \
-smp cpus=1 \
-m size=200M \
-display none \
-serial stdio \
-drive if=pflash,format=raw,readonly,file=target/x86_64-none/release/examples/bootrom
```