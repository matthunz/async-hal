cargo objcopy --release -- -O binary app.bin
st-flash --connect-under-reset --reset write app.bin 0x08000000
