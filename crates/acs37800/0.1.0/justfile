set dotenv-load	:= true
set dotenv-required := true

@_default:
    just --list

# Run the Raspberry Pi I2C async example remotely via SSH
run-remote-rpi-i2c:
    rsync -ave ssh --exclude target/ --exclude .git/ --exclude .env . $RPI_USER@$RPI_HOST:$RPI_PATH
    ssh $RPI_USER@$RPI_HOST "cd $RPI_PATH && $RPI_CARGO run --example raspberry-pi-i2c --no-default-features -F 'std linux-embedded-hal'"