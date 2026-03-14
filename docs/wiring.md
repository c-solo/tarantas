# Chassis Control System Wiring Schema

**Board:** STM32G431CBU6 "WeAct Black Pill"
**Development Language:** Rust (`embassy-stm32`)
**Architecture:** Differential Drive (Skid-Steer), 2 encoder channels.

---

### Global Rust Configuration

- tim8: timer 8 is used as embassy time driver (add time-driver-tim8 for embassy-stm32)

---

## Pinout Map

### 1. Main Computer Communication (Jetson Orin Nano)
*Protocol: UART (Serial). Baud-rate: 115200+. Device on Jetson: `/dev/ttyTHS1`.*

| STM32 Pin | Function  | Connection (Jetson Orin Nano) | Notes                        |
|:----------|:----------|:------------------------------|:-----------------------------|
| **PA2**   | USART2_TX | **UART1 RXD** (Pin 10)        | Data FROM Robot TO Jetson    |
| **PA3**   | USART2_RX | **UART1 TXD** (Pin 8)         | Data FROM Jetson TO Robot    |
| **GND**   | Ground    | **GND** (Pin 6/9/14)          | **MUST** share common ground |

*> Note: Jetson Orin Nano uses 3.3V logic on UART — same as STM32, no level shifter needed.*

### 2. Motors (Power Stage)
*Drivers: 2x BTS7960. Timers used in PWM Generation mode.*

| STM32 Pin | Timer/Channel | Rust Function | Connection (Drivers)                     |
|:----------|:--------------|:--------------|:-----------------------------------------|
| **PA6**   | TIM3_CH1      | `SimplePwm`   | **Left** BTS7960 -> **RPWM** (Forward)   |
| **PA7**   | TIM3_CH2      | `SimplePwm`   | **Left** BTS7960 -> **LPWM** (Backward)  |
| **PB6**   | TIM4_CH1      | `SimplePwm`   | **Right** BTS7960 -> **RPWM** (Forward)  |
| **PB7**   | TIM4_CH2      | `SimplePwm`   | **Right** BTS7960 -> **LPWM** (Backward) |
| **3.3V**  | Power         | -             | **R_EN** & **L_EN** (Both drivers)       |
| **3.3V**  | Power         | -             | **VCC** (Driver logic)                   |
| **GND**   | Ground        | -             | **GND** (Driver logic)                   |

*> Note: Connect driver power terminals (B+/B-) to the Motor Battery (4S LiPo, 14.8V). Connect M+/M- terminals to Motors.*

### 3. Encoders (Feedback)
*Wheel speed reading. Hardware timers used in QEI mode. Reading FRONT wheels only.*

| STM32 Pin | Timer    | Rust Function | Connection (Encoders)              |
|:----------|:---------|:--------------|:-----------------------------------|
| **PA8**   | TIM1_CH1 | `Qei`         | **Left** Front -> Phase **A**      |
| **PA9**   | TIM1_CH2 | `Qei`         | **Left** Front -> Phase **B**      |
| **PA0**   | TIM2_CH1 | `Qei`         | **Right** Front -> Phase **A**     |
| **PA1**   | TIM2_CH2 | `Qei`         | **Right** Front -> Phase **B**     |
| **3.3V**  | Power    | -             | Encoder **VCC** (Blue wire)        |
| **GND**   | Ground   | -             | Encoder **GND** (Black/Green wire) |

*> Warning: PA0 is the on-board KEY button. It will cease functioning as a button. Ensure encoders are powered by 3.3V
to avoid damaging the PA0 pin (it is not 5V-tolerant).*

### 4. Sensors (Navigation & Obstacle Avoidance)

- **BNO085** - 9-DOF IMU (Inertial Measurement Unit)
- **VL53L0X** - Distance Sensor 
- **VL6180X** - Cliff Sensor

*Bus: I2C1. Address Conflict Resolution: GPIO Control (XSHUT).*

All sensors share the I2C bus (PB8/PB9). Since VL53L0X and VL6180X share the default address `0x29`, 
their **XSHUT** pins must be connected to individual GPIOs. The MCU must enable them sequentially 
at startup to assign unique addresses (e.g., `0x30`, `0x31`...).

| STM32 Pin | Function | Device                   | Device Pin        |
|:----------|:---------|:-------------------------|:------------------|
| **PB8**   | I2C1_SCL | All Sensors              | **SCL**           |
| **PB9**   | I2C1_SDA | All Sensors              | **SDA**           |
| **3.3V**  | Power    | All Sensors              | **VIN / VCC**     |
| **GND**   | Ground   | All Sensors              | **GND**           |
| **PB0**   | GPIO_OUT | VL53L0X (Front Distance) | **XSHUT**         |
| **PB1**   | GPIO_OUT | VL53L0X (Back Distance)  | **XSHUT**         |
| **PB12**  | GPIO_OUT | VL6180X (Front Cliff)    | **XSHUT / GPIO0** |
| **PB13**  | GPIO_OUT | VL6180X (Back Cliff)     | **XSHUT / GPIO0** |

### 5. Power

*Dual-battery setup: separate power for motors and electronics to isolate motor noise from Jetson/sensors.*

| Supply              | Battery                  | Voltage Range    | Feeds                              |
|:--------------------|:-------------------------|:-----------------|:-----------------------------------|
| **Motor Battery**   | 4S LiPo 5200mAh 30C+    | 12.0–16.8V       | BTS7960 drivers (B+/B-)           |
| **Logic Battery**   | 4S LiPo 2200–3000mAh    | 12.0–16.8V       | Jetson DC barrel jack (7-20V), STM32 (via onboard 3.3V reg) |

**Common Ground:** Both battery GND **must** be connected to establish a shared voltage reference for UART.
Use a short, thick wire (14–16 AWG). If motor PWM noise causes UART glitches, add a ferrite toroid
on the GND bridge wire (3–5 turns through the core) to filter high-frequency interference while
preserving the DC ground reference.

**Charging:** Use a parallel charging board (both batteries must be identical 4S) to charge
from a single charger, or a dual-channel charger (e.g. ISDT D2) for independent balancing.

### 6. Miscellaneous (Debug & Status)

| STM32 Pin      | Purpose                                                              |
|:---------------|:---------------------------------------------------------------------|
| **PC13**       | On-board Blue LED (Active Low). Use for "Heartbeat" indication.      |
| **G, CLK, IO** | ST-Link V2 header (GND, SWCLK, SWDIO). For flashing and RTT logging. |

---
