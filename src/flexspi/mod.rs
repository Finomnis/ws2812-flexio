use imxrt_hal as hal;
use imxrt_ral as ral;

use ral::{flexspi, Valid};

/// Pins for the FlexSPI driver
pub struct Pins<Data0, Data1, Sclk, Ss0b> {
    /// Data0
    pub data0: Data0,
    /// Data1
    pub data1: Data1,
    /// Sclk
    pub sclk: Sclk,
    /// Ss0b
    pub ss0b: Ss0b,
}

/// An FlexSPI peripheral which is temporarily disabled.
pub struct Disabled<'a, const N: u8> {
    flexspi: &'a ral::flexspi::Instance<N>,
    disabled: bool,
}

impl<'a, const N: u8> Disabled<'a, N> {
    fn new(flexspi: &'a mut ral::flexspi::Instance<N>) -> Self {
        let disabled = ral::read_reg!(ral::flexspi, flexspi, MCR0, MDIS == 1);
        ral::modify_reg!(ral::flexspi, flexspi, MCR0, MDIS: 1);
        Self { flexspi, disabled }
    }

    /*
    /// Set the SPI mode for the peripheral
    pub fn set_mode(&mut self, mode: Mode) {
        // This could probably be changed when we're not disabled.
        // However, there's rules about when you can read TCR.
        // Specifically, reading TCR while it's being loaded from
        // the transmit FIFO could result in an incorrect reading.
        // Only permitting this when we're disabled might help
        // us avoid something troublesome.
        ral::modify_reg!(
            ral::lpspi,
            self.lpspi,
            TCR,
            CPOL: ((mode.polarity == Polarity::IdleHigh) as u32),
            CPHA: ((mode.phase == Phase::CaptureOnSecondTransition) as u32)
        );
    }

    /// Set the LPSPI clock speed (Hz).
    ///
    /// `source_clock_hz` is the LPSPI peripheral clock speed. To specify the
    /// peripheral clock, see the [`ccm::lpspi_clk`](crate::ccm::lpspi_clk) documentation.
    pub fn set_clock_hz(&mut self, source_clock_hz: u32, clock_hz: u32) {
        set_spi_clock(source_clock_hz, clock_hz, self.lpspi);
    }

    /// Set the watermark level for a given direction.
    ///
    /// Returns the watermark level committed to the hardware. This may be different
    /// than the supplied `watermark`, since it's limited by the hardware.
    ///
    /// When `direction == Direction::Rx`, the receive data flag is set whenever the
    /// number of words in the receive FIFO is greater than `watermark`.
    ///
    /// When `direction == Direction::Tx`, the transmit data flag is set whenever the
    /// the number of words in the transmit FIFO is less than, or equal, to `watermark`.
    #[inline]
    pub fn set_watermark(&mut self, direction: Direction, watermark: u8) -> u8 {
        let max_watermark = match direction {
            Direction::Rx => 1 << ral::read_reg!(ral::lpspi, self.lpspi, PARAM, RXFIFO),
            Direction::Tx => 1 << ral::read_reg!(ral::lpspi, self.lpspi, PARAM, TXFIFO),
        };

        let watermark = watermark.min(max_watermark - 1);

        match direction {
            Direction::Rx => {
                ral::modify_reg!(ral::lpspi, self.lpspi, FCR, RXWATER: watermark as u32)
            }
            Direction::Tx => {
                ral::modify_reg!(ral::lpspi, self.lpspi, FCR, TXWATER: watermark as u32)
            }
        }

        watermark
    }

    /// Set the sampling point of the LPSPI peripheral.
    ///
    /// When set to `SamplePoint::DelayedEdge`, the LPSPI will sample the input data
    /// on a delayed LPSPI_SCK edge, which improves the setup time when sampling data.
    #[inline]
    pub fn set_sample_point(&mut self, sample_point: SamplePoint) {
        match sample_point {
            SamplePoint::Edge => ral::modify_reg!(ral::lpspi, self.lpspi, CFGR1, SAMPLE: SAMPLE_0),
            SamplePoint::DelayedEdge => {
                ral::modify_reg!(ral::lpspi, self.lpspi, CFGR1, SAMPLE: SAMPLE_1)
            }
        }
    }
    */
}

impl<const N: u8> Drop for Disabled<'_, N> {
    fn drop(&mut self) {
        ral::modify_reg!(ral::flexspi, self.flexspi, MCR0, MDIS: self.disabled as u32);
    }
}

/// Driver for FlexSPI FIFO mode
pub struct FlexSPI<const N: u8>
where
    flexspi::Instance<N>: Valid,
{
    flexspi: flexspi::Instance<N>,
}

impl<const N: u8> FlexSPI<N>
where
    flexspi::Instance<N>: Valid,
{
    /// Initializes the FlexSPI driver
    pub fn init<Data0, Data1, Sclk, Ss0b>(
        ccm: &mut ral::ccm::CCM,
        flexspi: flexspi::Instance<N>,
        pins: Pins<Data0, Data1, Sclk, Ss0b>,
    ) -> Self
    where
        Data0: hal::iomuxc::flexspi::Pin<
            Module = hal::iomuxc::consts::Const<N>,
            Signal = hal::iomuxc::flexspi::Data0,
        >,
        Data1: hal::iomuxc::flexspi::Pin<
            Module = hal::iomuxc::consts::Const<N>,
            Signal = hal::iomuxc::flexspi::Data1,
        >,
        Sclk: hal::iomuxc::flexspi::Pin<
            Module = hal::iomuxc::consts::Const<N>,
            Signal = hal::iomuxc::flexspi::Sclk,
        >,
        Ss0b: hal::iomuxc::flexspi::Pin<
            Module = hal::iomuxc::consts::Const<N>,
            Signal = hal::iomuxc::flexspi::Ss0b,
        >,
    {
        // Setup clock
        match N {
            1 => ral::modify_reg!(ral::ccm, ccm,
                CSCMR1,
                FLEXSPI_CLK_SEL: FLEXSPI_CLK_SEL_2,
                FLEXSPI_PODF: FLEXSPI_PODF_7
            ),
            2 => ral::modify_reg!(ral::ccm, ccm,
                CBCMR,
                FLEXSPI2_CLK_SEL: FLEXSPI2_CLK_SEL_0,
                FLEXSPI2_PODF: FLEXSPI2_PODF_7
            ),
            _ => unreachable!(),
        };

        let mut this = Self { flexspi };

        // Enable and wait for reset finished
        ral::modify_reg!(ral::flexspi, this.flexspi, MCR0, MDIS: 0);
        while ral::read_reg!(ral::flexspi, this.flexspi, MCR0, SWRESET == 1) {}

        // Configure
        this.disabled(|disabled| {
            ral::modify_reg!(
                ral::flexspi,
                disabled.flexspi,
                MCR0,
                SCKFREERUNEN: SCKFREERUNEN_0,
                COMBINATIONEN: COMBINATIONEN_0,
                DOZEEN: DOZEEN_0,
                HSEN: HSEN_0,
                ATDFEN: ATDFEN_0,
                ARDFEN: ARDFEN_0,
                RXCLKSRC: RXCLKSRC_0
            );

            ral::write_reg!(
                ral::flexspi,
                disabled.flexspi,
                MCR1,
                SEQWAIT: 0xffff,
                AHBBUSWAIT: 0xffff
            );

            ral::modify_reg!(
                ral::flexspi,
                disabled.flexspi,
                MCR2,
                RESUMEWAIT: 0x2f,
                SCKBDIFFOPT: SCKBDIFFOPT_0,
                SAMEDEVICEEN: SAMEDEVICEEN_1,
                CLRAHBBUFOPT: CLRAHBBUFOPT_0
            );

            ral::modify_reg!(
                ral::flexspi,
                disabled.flexspi,
                DLLCR[0],
                OVRDVAL: 0,
                OVRDEN: 1,
                SLVDLYTARGET: 0,
                DLLRESET: 0,
                DLLEN: 0
            )
        });

        // Write LUTs
        for i in 0..this.flexspi.LUT.len() {
            ral::write_reg!(ral::flexspi, this.flexspi, LUT[i], 0);
        }
        ral::modify_reg!(
            ral::flexspi, this.flexspi, LUT[0],
            OPCODE0: 0x01, NUM_PADS0: 0x1, OPERAND0: 0x27,
            OPCODE1: 0x00, NUM_PADS1: 0x0, OPERAND1: 0x00
        );

        // Soft Reset to properly apply settings
        this.soft_reset();

        this
    }

    fn soft_reset(&mut self) {
        ral::modify_reg!(ral::flexspi, self.flexspi, MCR0, SWRESET: 1);
        while ral::read_reg!(ral::flexspi, self.flexspi, MCR0, SWRESET == 1) {}
    }

    /// A dummy function for development purposes
    pub fn dummy_write(&mut self) {
        /*
            From reference manual:

            1. Fill IP TX FIFO with programming data if this is a programing command (for
            instance, programming flash data and flash status registers.)
            2. Set flash access start address (IPCR0[SFAR]).
            3. Write the IPCR1 with the read/program data size, sequence index, and sequence
            number (IPCR1[IDATSZ, ISEQID, and ISEQNUM]).
            4. Trigger flash access command by writing 1 to IPCMD[TRG].
            5. Wait for the INTR[IPCMDDONE] bit to set or for the IP command done interrupt to
            fire indicating the command has completed on the FlexSPI interface.
        */

        let lut_seq_range = 0u32..=0;

        // Setup parameters
        ral::write_reg!(
            ral::flexspi, self.flexspi, IPCR0,
            SFAR: 0
        );
        ral::write_reg!(
            ral::flexspi, self.flexspi, IPCR1,
            IPAREN: 0,
            ISEQID: lut_seq_range.start(),
            ISEQNUM: (lut_seq_range.count() as u32) - 1,
            IDATSZ: 0
        );

        // Reset interrupt flags
        ral::modify_reg!(
            ral::flexspi, self.flexspi, INTR,
            IPCMDDONE: 0,
            IPCMDERR: 0,
            IPCMDGE: 0
        );

        // Start!
        ral::write_reg!(
            ral::flexspi, self.flexspi, IPCMD,
            TRG: 1
        );

        // Wait for done
        while ral::read_reg!(ral::flexspi, self.flexspi, INTR, IPCMDDONE == 0) {}

        // Error handling
        let err = if ral::read_reg!(ral::flexspi, self.flexspi, INTR, IPCMDERR == 1) {
            Some(ral::read_reg!(
                ral::flexspi,
                self.flexspi,
                STS1,
                IPCMDERRCODE
            ))
        } else {
            None
        };

        // Reset interrupt flags
        ral::modify_reg!(
            ral::flexspi, self.flexspi, INTR,
            IPCMDDONE: 0,
            IPCMDERR: 0,
            IPCMDGE: 0
        );

        if let Some(errcode) = err {
            log::error!("FlexSPI write failed! Error code: {}", errcode);
        }
    }

    /// Temporarily disable the FlexSPI peripheral.
    ///
    /// The handle to a [`Disabled`](crate::flexspi::Disabled) driver lets you modify
    /// FlexSPI settings that require a fully disabled peripheral. This will clear the transmit
    /// and receive FIFOs.
    pub fn disabled<R>(&mut self, func: impl FnOnce(&mut Disabled<N>) -> R) -> R {
        // TODO: cancel/await existing transfers
        let mut disabled = Disabled::new(&mut self.flexspi);
        func(&mut disabled)
    }
}
