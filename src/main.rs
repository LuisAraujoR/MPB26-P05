mod lcd;

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{InterruptType, PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::sys::EspError;
use std::sync::atomic::{AtomicBool, Ordering};

use lcd::Lcd;

const NORMAL: u8 = 0;
const AJUSTAR_HOR: u8 = 1;
const AJUSTAR_MIN: u8 = 2;
const AJUSTAR_SEG: u8 = 3;

// Banderas atómicas para comunicar las interrupciones de los botones
static BOTON_1_PRESIONADO: AtomicBool = AtomicBool::new(false);
static BOTON_2_PRESIONADO: AtomicBool = AtomicBool::new(false);
static BOTON_3_PRESIONADO: AtomicBool = AtomicBool::new(false);
static BOTON_4_PRESIONADO: AtomicBool = AtomicBool::new(false);

fn main() -> Result<(), EspError> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let mut hor: u8 = 16;
    let mut min: u8 = 30;
    let mut seg: u8 = 15;

    let peripherals = Peripherals::take()?;

    let rs = PinDriver::output(peripherals.pins.gpio15)?;
    let en = PinDriver::output(peripherals.pins.gpio16)?;
    let d4 = PinDriver::output(peripherals.pins.gpio17)?;
    let d5 = PinDriver::output(peripherals.pins.gpio18)?;
    let d6 = PinDriver::output(peripherals.pins.gpio8)?;
    let d7 = PinDriver::output(peripherals.pins.gpio3)?;

    // Configuración de los botones con pull-up e interrupción por flanco de bajada
    let mut boton1 = PinDriver::input(peripherals.pins.gpio4, Pull::Up)?;
    let mut boton2 = PinDriver::input(peripherals.pins.gpio5, Pull::Up)?;
    let mut boton3 = PinDriver::input(peripherals.pins.gpio6, Pull::Up)?;
    let mut boton4 = PinDriver::input(peripherals.pins.gpio7, Pull::Up)?;

    boton1.set_interrupt_type(InterruptType::NegEdge)?;
    boton2.set_interrupt_type(InterruptType::NegEdge)?;
    boton3.set_interrupt_type(InterruptType::NegEdge)?;
    boton4.set_interrupt_type(InterruptType::NegEdge)?;

    unsafe {
        boton1.subscribe(|| {
            BOTON_1_PRESIONADO.store(true, Ordering::Relaxed);
        })?;
        boton2.subscribe(|| {
            BOTON_2_PRESIONADO.store(true, Ordering::Relaxed);
        })?;
        boton3.subscribe(|| {
            BOTON_3_PRESIONADO.store(true, Ordering::Relaxed);
        })?;
        boton4.subscribe(|| {
            BOTON_4_PRESIONADO.store(true, Ordering::Relaxed);
        })?;
    }

    boton1.enable_interrupt()?;
    boton2.enable_interrupt()?;
    boton3.enable_interrupt()?;
    boton4.enable_interrupt()?;

    let mut lcd = Lcd::new(rs, en, d4, d5, d6, d7)?;
    lcd.clear()?;
    log::info!("LCD inicializada correctamente.");

    let mut modo: u8 = NORMAL;

    lcd.set_cursor(0, 0)?;
    let tiempo = format!("Hora:   {:02}:{:02}:{:02}", hor, min, seg);
    write!(&mut lcd, "{}", tiempo)?;

    lcd.set_cursor(0, 1)?;
    write!(&mut lcd, "Modo:   NORMAL")?;

    let mut contador = 0;

    loop {
        FreeRtos::delay_ms(50);
        contador += 1;
        if contador == 20 {
            contador = 0;
            seg += 1;
            if seg == 60 {
                seg = 0;
                min += 1;
                if min == 60 {
                    min = 0;
                    hor += 1;
                    if hor == 24 {
                        hor = 0;
                    }
                }
            }

            log::info!("{:02}:{:02}:{:02}", hor, min, seg);
            lcd.set_cursor(0, 0)?;
            let tiempo = format!("Hora:   {:02}:{:02}:{:02}", hor, min, seg);
            write!(&mut lcd, "{}", tiempo)?;
        }

        if BOTON_1_PRESIONADO.swap(false, Ordering::Relaxed) {
            modo = (modo + 1) % 4;
            lcd.set_cursor(0, 1)?;

            match modo {
                NORMAL => {
                    write!(&mut lcd, "Modo:   NORMAL")?;
                }
                AJUSTAR_HOR => {
                    write!(&mut lcd, "Modo:   HOR   ")?;
                }
                AJUSTAR_MIN => {
                    write!(&mut lcd, "Modo:   MIN   ")?;
                }
                _ => {
                    write!(&mut lcd, "Modo:   SEG   ")?;
                }
            }
            FreeRtos::delay_ms(150);
            boton1.enable_interrupt()?;
        }

        if BOTON_2_PRESIONADO.swap(false, Ordering::Relaxed) {
            match modo {
                AJUSTAR_HOR => {
                    hor = (hor + 1) % 24;
                    lcd.set_cursor(8, 0)?;
                    write!(&mut lcd, "{}", hor)?;
                }
                AJUSTAR_MIN => {
                    min = (min + 1) % 60;
                    lcd.set_cursor(11, 0)?;
                    write!(&mut lcd, "{}", min)?;
                }
                AJUSTAR_SEG => {
                    seg = (seg + 1) % 60;
                    lcd.set_cursor(14, 0)?;
                    write!(&mut lcd, "{}", seg)?;
                }
                _ => {}
            }
            FreeRtos::delay_ms(150);
            boton2.enable_interrupt()?;
        }

        if BOTON_3_PRESIONADO.swap(false, Ordering::Relaxed) {
            match modo {
                AJUSTAR_HOR => {
                    if hor > 0 {
                        hor -= 1;
                    } else {
                        hor = 23;
                    }
                    lcd.set_cursor(8, 0)?;
                    write!(&mut lcd, "{}", hor)?;
                }
                AJUSTAR_MIN => {
                    if min > 0 {
                        min -= 1;
                    } else {
                        min = 59;
                    }
                    lcd.set_cursor(11, 0)?;
                    write!(&mut lcd, "{}", min)?;
                }
                AJUSTAR_SEG => {
                    if seg > 0 {
                        seg -= 1;
                    } else {
                        seg = 59;
                    }
                    lcd.set_cursor(14, 0)?;
                    write!(&mut lcd, "{}", seg)?;
                }
                _ => {}
            }
            FreeRtos::delay_ms(150);
            boton3.enable_interrupt()?;
        }
    }
}
