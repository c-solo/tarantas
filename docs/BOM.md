# BOM (Bill of Materials)

## Compute & Communication

- [x] Jetson Orin Nano (8GB) + active cooling (fan included with carrier board).
- [x] STM32F401CCU6 "Black Pill" — MCU for motor and sensor control.
- [x] NVMe SSD: M.2 2230/2242 for Jetson (no microSD needed — boots from NVMe).

## Drivetrain

- [x] Chassis: TR300-P tracked tank (300×240×125mm, aluminum, 5kg load capacity, suspension).
- [ ] Motors: 2x 37GB520 (12V, 200 RPM, 1:30 gear ratio) with Hall encoders.
- [ ] Drivers: 1x BTS7960 (IBT-2) 43A.
- [ ] ~~PWM expander: PCA9685~~ — not needed, 2 motors driven directly from STM32 PWM.

## Sensors & Peripherals

- [ ] IMU: 1x Adafruit BNO085 (or SparkFun VR IMU). Don't cheap out here.
- [ ] Distance: 2-3x VL53L0X (ToF Distance Sensor).
- [ ] Camera: CSI camera (IMX219/IMX477) or USB webcam. Jetson supports MIPI CSI-2 with hardware ISP.

## Power (Dual Battery)

- [ ] Motor Battery: LiPo 4S (14.8V) 5200mAh 30C+ (XT60 connector). Powers motors via BTS7960.
- [ ] Logic Battery: LiPo 4S (14.8V) 2200-3000mAh 10C+ (XT60 connector). Powers Jetson (DC barrel jack 7-20V) and STM32.
- [ ] GND bridge: thick 14-16 AWG wire between both battery GNDs. Optional ferrite toroid to filter motor PWM noise.
- [ ] Parallel charging board (4S) or dual-channel charger for simultaneous charging.
- [ ] Wires: Dupont jumper cables, 14-16 AWG power wires.
- [ ] Connectors: XT60 (male/female) for soldering power lines.
