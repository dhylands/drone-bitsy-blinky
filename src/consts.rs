//! Project constants.

/// FLASH Latency
//
// 5 for Vcc = 3.3v and 150 MHz <= HCLK <= 168 MHz
// Power on default is 0.
pub const FLASH_LATENCY: u32 = 5;

/// HSE crystal frequency.
pub const HSE_FREQ: u32 = 25_000_000;

/// HSI crystal frequency.
pub const HSI_FREQ: u32 = 8_000_000;

// VCO-freq = HSE * (PLLN / PLLM) = 336 MHz
// PLL general clock = VCO-freq / PLLP = 168 MHz
// USB/SDIO/RG freq = VCO-freq / PLLQ = 48 MHz

/// PLLM - Division factor for the main PLL (PLL) input clock
pub const PLLM: u32 = 25;

/// PLLN - Main PLL multiplication factor for VCO
pub const PLLN: u32 = 336;

/// PLLP - Main PLL division factor for main system clock
pub const PLLP: u32 = 0b00; // 0b00 = Divide by 2

/// PLLQ - Main PLL division factor for USB OTG FS, and SDIO clocks
pub const PLLQ: u32 = 0b0111; // 0b0100 - Divide by 7

/// System clock frequency (should be 168 MHz)
pub const SYS_CLK: u32 = ((HSE_FREQ / PLLM) * PLLN) / ((PLLP + 1) * 2);

/// SysTick clock frequency.
pub const SYS_TICK_FREQ: u32 = SYS_CLK / 8;
