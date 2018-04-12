use std::collections::HashSet;

#[derive(Debug, Deserialize, Serialize)]
pub struct State {
    pub interconnect: InterconnectState,
    pub cpu: V810State,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InterconnectState {
    pub rom: Box<[u8]>,
    pub wram: Box<[u8]>,
    pub sram: SramState,
    pub vip: VipState,
    pub vsu: VsuState,
    pub timer: TimerState,
    pub game_pad: GamePadState,
    pub com_port: ComPortState,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ComPortState {
    pub cdtr: u8,
    pub cdrr: u8,

    pub c_stat: bool,

    pub transfer_bit_index: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GamePadState {
    pub a_pressed: bool,
    pub b_pressed: bool,
    pub start_pressed: bool,
    pub select_pressed: bool,
    pub l_pressed: bool,
    pub r_pressed: bool,
    pub left_d_pad_up_pressed: bool,
    pub left_d_pad_down_pressed: bool,
    pub left_d_pad_left_pressed: bool,
    pub left_d_pad_right_pressed: bool,
    pub right_d_pad_up_pressed: bool,
    pub right_d_pad_down_pressed: bool,
    pub right_d_pad_left_pressed: bool,
    pub right_d_pad_right_pressed: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SramState {
    pub bytes: Box<[u8]>,

    pub size: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum IntervalState {
    Large,
    Small,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TimerState {
    pub t_clk_sel: IntervalState,
    pub tim_z_int: bool,
    pub z_stat: bool,
    pub t_enb: bool,
    pub reload: u16,
    pub counter: u16,

    pub tick_counter: u32,
    pub zero_interrupt: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum DisplayStateState {
    Idle,
    LeftFramebuffer,
    RightFramebuffer,
    Finished,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum DrawingStateState {
    Idle,
    Drawing,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VipState {
    pub vram: Box<[u8]>,

    pub display_state: DisplayStateState,

    pub drawing_state: DrawingStateState,

    pub reg_intpnd_lfbend: bool,
    pub reg_intpnd_rfbend: bool,
    pub reg_intpnd_gamestart: bool,
    pub reg_intpnd_framestart: bool,
    pub reg_intpnd_sbhit: bool,
    pub reg_intpnd_xpend: bool,

    pub reg_intenb_lfbend: bool,
    pub reg_intenb_rfbend: bool,
    pub reg_intenb_gamestart: bool,
    pub reg_intenb_framestart: bool,
    pub reg_intenb_sbhit: bool,
    pub reg_intenb_xpend: bool,

    pub reg_dpctrl_disp: bool,
    pub reg_dpctrl_synce: bool,

    pub reg_xpctrl_xpen: bool,
    pub reg_xpctrl_sbcount: u32,
    pub reg_xpctrl_sbcmp: u32,
    pub reg_xpctrl_sbout: bool,

    pub reg_frmcyc: u32,

    pub reg_brta: u8,
    pub reg_brtb: u8,
    pub reg_brtc: u8,

    pub reg_spt0: u16,
    pub reg_spt1: u16,
    pub reg_spt2: u16,
    pub reg_spt3: u16,

    pub reg_gplt0: u8,
    pub reg_gplt1: u8,
    pub reg_gplt2: u8,
    pub reg_gplt3: u8,

    pub reg_jplt0: u8,
    pub reg_jplt1: u8,
    pub reg_jplt2: u8,
    pub reg_jplt3: u8,

    pub reg_bkcol: u8,

    pub display_frame_eighth_clock_counter: u32,
    pub display_frame_eighth_counter: u32,

    pub drawing_block_counter: u32,
    pub drawing_sbout_counter: u32,

    pub fclk: u32,

    pub display_first_framebuffers: bool,
    pub last_bkcol: u8,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IntRegState {
    pub output_enable: bool,
    pub interval_data: bool,
    pub interval_counter_setting_values: u32,

    pub interval_counter: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LrvRegState {
    pub left: u32,
    pub right: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EnvelopeState {
    pub reg_data_reload: u32,
    pub reg_data_direction: bool,
    pub reg_data_step_interval: u32,

    pub reg_control_repeat: bool,
    pub reg_control_enable: bool,

    pub level: u32,

    pub envelope_counter: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StandardSoundState {
    pub reg_int: IntRegState,

    pub reg_lrv: LrvRegState,

    pub fql: u32,
    pub fqh: u32,

    pub envelope: EnvelopeState,

    pub ram: u32,

    pub frequency_counter: u32,
    pub phase: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SweepModSoundState {
    pub reg_int: IntRegState,

    pub reg_lrv: LrvRegState,

    pub fql: u32,
    pub fqh: u32,
    pub frequency_low: u32,
    pub frequency_high: u32,
    pub next_frequency_low: u32,
    pub next_frequency_high: u32,

    pub envelope: EnvelopeState,

    pub reg_sweep_mod_enable: bool,
    pub reg_mod_repeat: bool,
    pub reg_function: bool,

    pub reg_sweep_mod_base_interval: bool,
    pub reg_sweep_mod_interval: u32,
    pub reg_sweep_direction: bool,
    pub reg_sweep_shift_amount: u32,

    pub ram: u32,

    pub frequency_counter: u32,
    pub phase: u32,

    pub sweep_mod_counter: u32,
    pub mod_phase: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NoiseSoundState {
    pub reg_int: IntRegState,

    pub reg_lrv: LrvRegState,

    pub fql: u32,
    pub fqh: u32,

    pub envelope: EnvelopeState,

    pub reg_noise_control: u32,

    pub frequency_counter: u32,
    pub shift: u32,
    pub output: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VsuState {
    pub waveform_data: Box<[u8]>,
    pub mod_data: Box<[i8]>,

    pub sound1: StandardSoundState,
    pub sound2: StandardSoundState,
    pub sound3: StandardSoundState,
    pub sound4: StandardSoundState,
    pub sound5: SweepModSoundState,
    pub sound6: NoiseSoundState,

    pub duration_clock_counter: u32,
    pub envelope_clock_counter: u32,
    pub frequency_clock_counter: u32,
    pub sweep_mod_clock_counter: u32,
    pub noise_clock_counter: u32,
    pub sample_clock_counter: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct V810State {
    pub reg_pc: u32,

    pub reg_gpr: Box<[u32; 32]>,

    pub reg_eipc: u32,
    pub reg_eipsw: u32,
    pub reg_ecr: u16,
    pub reg_fepc: u32,
    pub reg_fepsw: u32,

    pub psw_zero: bool,
    pub psw_sign: bool,
    pub psw_overflow: bool,
    pub psw_carry: bool,
    pub psw_fp_precision_degredation: bool,
    pub psw_fp_underflow: bool,
    pub psw_fp_overflow: bool,
    pub psw_fp_zero_division: bool,
    pub psw_fp_invalid_operation: bool,
    pub psw_fp_reserved_operand: bool,
    pub psw_interrupt_disable: bool,
    pub psw_address_trap_enable: bool,
    pub psw_exception_pending: bool,
    pub psw_nmi_pending: bool,
    pub psw_interrupt_mask_level: u32,

    pub is_halted: bool,

    pub cache: CacheState,

    pub watchpoints: HashSet<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CacheState {
    pub hits: u64,
    pub misses: u64,
    pub is_enabled: bool,
    pub entries: Box<[CacheEntryState]>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CacheEntryState {
    pub tag: u32,
    pub base_addr: u32,
    pub subblock_valid: [bool; 2],
}
