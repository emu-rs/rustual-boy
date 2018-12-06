extern crate rustual_boy_core;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_bytes;
extern crate bincode;

extern crate lz4;

pub mod version1;
mod serde_ibytes;

use lz4::{EncoderBuilder, Decoder};

use rustual_boy_core::timer::*;
use rustual_boy_core::vip::*;
use rustual_boy_core::virtual_boy::*;
use rustual_boy_core::vsu::*;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum VersionedState {
    Version1(version1::State),
}

pub type State = version1::State;

pub fn serialize(state: State) -> Result<Vec<u8>, String> {
    let versioned_state = VersionedState::Version1(state);
    let encoded = Vec::with_capacity(512 * 1024);
    let mut encoder = EncoderBuilder::new()
        .level(2)
        .block_size(lz4::BlockSize::Max1MB)
        .build(encoded)
        .map_err(|e| format!("Couldn't build lz4 encoder: {}", e))?;
    bincode::serialize_into(&mut encoder, &versioned_state).map_err(|e| format!("Couldn't save state: {}", e))?;
    let (encoded, result) = encoder.finish();
    if let Err(e) = result {
        return Err(format!("Couldn't finish lz4 encoding: {}", e));
    }
    Ok(encoded)
}

pub fn deserialize(encoded: &[u8]) -> Result<State, String> {
    let decoder = Decoder::new(encoded).map_err(|e| format!("Couldn't build lz4 decoder: {}", e))?;
    let versioned_state = bincode::deserialize_from(decoder).map_err(|e| format!("Couldn't deserialize state: {}", e))?;
    Ok(match versioned_state {
        VersionedState::Version1(state) => state,
    })
}

pub fn get_state(vb: &VirtualBoy) -> State {
    State {
        interconnect: version1::InterconnectState {
            rom: vb.interconnect.rom.bytes.clone(),
            wram: vb.interconnect.wram.bytes.clone(),
            sram: vb.interconnect.sram.bytes[..vb.interconnect.sram.size].to_vec().into_boxed_slice(),
            vip: version1::VipState {
                vram: vb.interconnect.vip.vram.clone(),

                display_state: match vb.interconnect.vip.display_state {
                    DisplayState::Idle => version1::DisplayStateState::Idle,
                    DisplayState::LeftFramebuffer => version1::DisplayStateState::LeftFramebuffer,
                    DisplayState::RightFramebuffer => version1::DisplayStateState::RightFramebuffer,
                    DisplayState::Finished => version1::DisplayStateState::Finished,
                },

                drawing_state: match vb.interconnect.vip.drawing_state {
                    DrawingState::Idle => version1::DrawingStateState::Idle,
                    DrawingState::Drawing => version1::DrawingStateState::Drawing,
                },

                reg_intpnd_lfbend: vb.interconnect.vip.reg_intpnd_lfbend,
                reg_intpnd_rfbend: vb.interconnect.vip.reg_intpnd_rfbend,
                reg_intpnd_gamestart: vb.interconnect.vip.reg_intpnd_gamestart,
                reg_intpnd_framestart: vb.interconnect.vip.reg_intpnd_framestart,
                reg_intpnd_sbhit: vb.interconnect.vip.reg_intpnd_sbhit,
                reg_intpnd_xpend: vb.interconnect.vip.reg_intpnd_xpend,

                reg_intenb_lfbend: vb.interconnect.vip.reg_intenb_lfbend,
                reg_intenb_rfbend: vb.interconnect.vip.reg_intenb_rfbend,
                reg_intenb_gamestart: vb.interconnect.vip.reg_intenb_gamestart,
                reg_intenb_framestart: vb.interconnect.vip.reg_intenb_framestart,
                reg_intenb_sbhit: vb.interconnect.vip.reg_intenb_sbhit,
                reg_intenb_xpend: vb.interconnect.vip.reg_intenb_xpend,

                reg_dpctrl_disp: vb.interconnect.vip.reg_dpctrl_disp,
                reg_dpctrl_synce: vb.interconnect.vip.reg_dpctrl_synce,

                reg_xpctrl_xpen: vb.interconnect.vip.reg_xpctrl_xpen,
                reg_xpctrl_sbcount: vb.interconnect.vip.reg_xpctrl_sbcount,
                reg_xpctrl_sbcmp: vb.interconnect.vip.reg_xpctrl_sbcmp,
                reg_xpctrl_sbout: vb.interconnect.vip.reg_xpctrl_sbout,

                reg_frmcyc: vb.interconnect.vip.reg_frmcyc,

                reg_brta: vb.interconnect.vip.reg_brta,
                reg_brtb: vb.interconnect.vip.reg_brtb,
                reg_brtc: vb.interconnect.vip.reg_brtc,

                reg_spt0: vb.interconnect.vip.reg_spt0,
                reg_spt1: vb.interconnect.vip.reg_spt1,
                reg_spt2: vb.interconnect.vip.reg_spt2,
                reg_spt3: vb.interconnect.vip.reg_spt3,

                reg_gplt0: vb.interconnect.vip.reg_gplt0,
                reg_gplt1: vb.interconnect.vip.reg_gplt1,
                reg_gplt2: vb.interconnect.vip.reg_gplt2,
                reg_gplt3: vb.interconnect.vip.reg_gplt3,

                reg_jplt0: vb.interconnect.vip.reg_jplt0,
                reg_jplt1: vb.interconnect.vip.reg_jplt1,
                reg_jplt2: vb.interconnect.vip.reg_jplt2,
                reg_jplt3: vb.interconnect.vip.reg_jplt3,

                reg_bkcol: vb.interconnect.vip.reg_bkcol,

                display_frame_eighth_clock_counter: vb.interconnect.vip.display_frame_eighth_clock_counter,
                display_frame_eighth_counter: vb.interconnect.vip.display_frame_eighth_counter,

                drawing_block_counter: vb.interconnect.vip.drawing_block_counter,
                drawing_sbout_counter: vb.interconnect.vip.drawing_sbout_counter,

                fclk: vb.interconnect.vip.fclk,

                display_first_framebuffers: vb.interconnect.vip.display_first_framebuffers,
                last_bkcol: vb.interconnect.vip.last_bkcol,
            },
            vsu: version1::VsuState {
                waveform_data: vb.interconnect.vsu.waveform_data.clone(),
                mod_data: vb.interconnect.vsu.mod_data.clone(),

                sound1: get_standard_sound_state(&vb.interconnect.vsu.sound1),
                sound2: get_standard_sound_state(&vb.interconnect.vsu.sound2),
                sound3: get_standard_sound_state(&vb.interconnect.vsu.sound3),
                sound4: get_standard_sound_state(&vb.interconnect.vsu.sound4),
                sound5: version1::SweepModSoundState {
                    reg_int: get_int_reg_state(&vb.interconnect.vsu.sound5.reg_int),

                    reg_lrv: get_lrv_reg_state(&vb.interconnect.vsu.sound5.reg_lrv),

                    fql: vb.interconnect.vsu.sound5.fql,
                    fqh: vb.interconnect.vsu.sound5.fqh,
                    frequency_low: vb.interconnect.vsu.sound5.frequency_low,
                    frequency_high: vb.interconnect.vsu.sound5.frequency_high,
                    next_frequency_low: vb.interconnect.vsu.sound5.next_frequency_low,
                    next_frequency_high: vb.interconnect.vsu.sound5.next_frequency_high,

                    envelope: get_envelope_state(&vb.interconnect.vsu.sound5.envelope),

                    reg_sweep_mod_enable: vb.interconnect.vsu.sound5.reg_sweep_mod_enable,
                    reg_mod_repeat: vb.interconnect.vsu.sound5.reg_mod_repeat,
                    reg_function: vb.interconnect.vsu.sound5.reg_function,

                    reg_sweep_mod_base_interval: vb.interconnect.vsu.sound5.reg_sweep_mod_base_interval,
                    reg_sweep_mod_interval: vb.interconnect.vsu.sound5.reg_sweep_mod_interval,
                    reg_sweep_direction: vb.interconnect.vsu.sound5.reg_sweep_direction,
                    reg_sweep_shift_amount: vb.interconnect.vsu.sound5.reg_sweep_shift_amount,

                    ram: vb.interconnect.vsu.sound5.ram,

                    frequency_counter: vb.interconnect.vsu.sound5.frequency_counter,
                    phase: vb.interconnect.vsu.sound5.phase,

                    sweep_mod_counter: vb.interconnect.vsu.sound5.sweep_mod_counter,
                    mod_phase: vb.interconnect.vsu.sound5.mod_phase,
                },
                sound6: version1::NoiseSoundState {
                    reg_int: get_int_reg_state(&vb.interconnect.vsu.sound6.reg_int),

                    reg_lrv: get_lrv_reg_state(&vb.interconnect.vsu.sound6.reg_lrv),

                    fql: vb.interconnect.vsu.sound6.fql,
                    fqh: vb.interconnect.vsu.sound6.fqh,

                    envelope: get_envelope_state(&vb.interconnect.vsu.sound6.envelope),

                    reg_noise_control: vb.interconnect.vsu.sound6.reg_noise_control,

                    frequency_counter: vb.interconnect.vsu.sound6.frequency_counter,
                    shift: vb.interconnect.vsu.sound6.shift,
                    output: vb.interconnect.vsu.sound6.output,
                },

                duration_clock_counter: vb.interconnect.vsu.duration_clock_counter,
                envelope_clock_counter: vb.interconnect.vsu.envelope_clock_counter,
                frequency_clock_counter: vb.interconnect.vsu.frequency_clock_counter,
                sweep_mod_clock_counter: vb.interconnect.vsu.sweep_mod_clock_counter,
                noise_clock_counter: vb.interconnect.vsu.noise_clock_counter,
                sample_clock_counter: vb.interconnect.vsu.sample_clock_counter,
            },
            timer: version1::TimerState {
                t_clk_sel: match vb.interconnect.timer.t_clk_sel {
                    Interval::Large => version1::IntervalState::Large,
                    Interval::Small => version1::IntervalState::Small,
                },
                tim_z_int: vb.interconnect.timer.tim_z_int,
                z_stat: vb.interconnect.timer.z_stat,
                t_enb: vb.interconnect.timer.t_enb,
                reload: vb.interconnect.timer.reload,
                counter: vb.interconnect.timer.counter,

                tick_counter: vb.interconnect.timer.tick_counter,
                zero_interrupt: vb.interconnect.timer.zero_interrupt,
            },
            game_pad: version1::GamePadState {
                a_pressed: vb.interconnect.game_pad.a_pressed,
                b_pressed: vb.interconnect.game_pad.b_pressed,
                start_pressed: vb.interconnect.game_pad.start_pressed,
                select_pressed: vb.interconnect.game_pad.select_pressed,
                l_pressed: vb.interconnect.game_pad.l_pressed,
                r_pressed: vb.interconnect.game_pad.r_pressed,
                left_d_pad_up_pressed: vb.interconnect.game_pad.left_d_pad_up_pressed,
                left_d_pad_down_pressed: vb.interconnect.game_pad.left_d_pad_down_pressed,
                left_d_pad_left_pressed: vb.interconnect.game_pad.left_d_pad_left_pressed,
                left_d_pad_right_pressed: vb.interconnect.game_pad.left_d_pad_right_pressed,
                right_d_pad_up_pressed: vb.interconnect.game_pad.right_d_pad_up_pressed,
                right_d_pad_down_pressed: vb.interconnect.game_pad.right_d_pad_down_pressed,
                right_d_pad_left_pressed: vb.interconnect.game_pad.right_d_pad_left_pressed,
                right_d_pad_right_pressed: vb.interconnect.game_pad.right_d_pad_right_pressed,
            },
            com_port: version1::ComPortState {
                cdtr: vb.interconnect.com_port.cdtr,
                cdrr: vb.interconnect.com_port.cdrr,

                c_stat: vb.interconnect.com_port.c_stat,

                transfer_bit_index: vb.interconnect.com_port.transfer_bit_index,
            },
        },
        cpu: version1::V810State {
            reg_pc: vb.cpu.reg_pc,

            reg_gpr: vb.cpu.reg_gpr.clone(),

            reg_eipc: vb.cpu.reg_eipc,
            reg_eipsw: vb.cpu.reg_eipsw,
            reg_ecr: vb.cpu.reg_ecr,
            reg_fepc: vb.cpu.reg_fepc,
            reg_fepsw: vb.cpu.reg_fepsw,

            psw_zero: vb.cpu.psw_zero,
            psw_sign: vb.cpu.psw_sign,
            psw_overflow: vb.cpu.psw_overflow,
            psw_carry: vb.cpu.psw_carry,
            psw_fp_precision_degredation: vb.cpu.psw_fp_precision_degredation,
            psw_fp_underflow: vb.cpu.psw_fp_underflow,
            psw_fp_overflow: vb.cpu.psw_fp_overflow,
            psw_fp_zero_division: vb.cpu.psw_fp_zero_division,
            psw_fp_invalid_operation: vb.cpu.psw_fp_invalid_operation,
            psw_fp_reserved_operand: vb.cpu.psw_fp_reserved_operand,
            psw_interrupt_disable: vb.cpu.psw_interrupt_disable,
            psw_address_trap_enable: vb.cpu.psw_address_trap_enable,
            psw_exception_pending: vb.cpu.psw_exception_pending,
            psw_nmi_pending: vb.cpu.psw_nmi_pending,
            psw_interrupt_mask_level: vb.cpu.psw_interrupt_mask_level,

            is_halted: vb.cpu.is_halted,

            cache: version1::CacheState {
                hits: vb.cpu.cache.hits,
                misses: vb.cpu.cache.misses,
                is_enabled: vb.cpu.cache.is_enabled,
                entries: vb.cpu.cache.entries.iter().map(|entry| {
                    version1::CacheEntryState {
                        tag: entry.tag,
                        base_addr: entry.base_addr,
                        subblock_valid: entry.subblock_valid.clone(),
                    }
                }).collect::<Vec<_>>().into_boxed_slice(),
            },

            watchpoints: vb.cpu.watchpoints.clone(),
        },
    }
}

fn get_standard_sound_state(sound: &StandardSound) -> version1::StandardSoundState {
    version1::StandardSoundState {
        reg_int: get_int_reg_state(&sound.reg_int),

        reg_lrv: get_lrv_reg_state(&sound.reg_lrv),

        fql: sound.fql,
        fqh: sound.fqh,

        envelope: get_envelope_state(&sound.envelope),

        ram: sound.ram,

        frequency_counter: sound.frequency_counter,
        phase: sound.phase,
    }
}

fn get_int_reg_state(int_reg: &IntReg) -> version1::IntRegState {
    version1::IntRegState {
        output_enable: int_reg.output_enable,
        interval_data: int_reg.interval_data,
        interval_counter_setting_values: int_reg.interval_counter_setting_values,

        interval_counter: int_reg.interval_counter,
    }
}

fn get_lrv_reg_state(lrv_reg: &LrvReg) -> version1::LrvRegState {
    version1::LrvRegState {
        left: lrv_reg.left,
        right: lrv_reg.right,
    }
}

fn get_envelope_state(envelope: &Envelope) -> version1::EnvelopeState {
    version1::EnvelopeState {
        reg_data_reload: envelope.reg_data_reload,
        reg_data_direction: envelope.reg_data_direction,
        reg_data_step_interval: envelope.reg_data_step_interval,

        reg_control_repeat: envelope.reg_control_repeat,
        reg_control_enable: envelope.reg_control_enable,

        level: envelope.level,

        envelope_counter: envelope.envelope_counter,
    }
}

pub fn apply(vb: &mut VirtualBoy, state: &State) {
    // Rom
    vb.interconnect.rom.bytes[..state.interconnect.rom.len()].copy_from_slice(&state.interconnect.rom);
    vb.interconnect.rom.size = state.interconnect.rom.len();

    // Wram
    vb.interconnect.wram.bytes.copy_from_slice(&state.interconnect.wram);

    // Sram
    vb.interconnect.sram.bytes[..state.interconnect.sram.len()].copy_from_slice(&state.interconnect.sram);
    vb.interconnect.sram.size = state.interconnect.sram.len();

    // Vip
    vb.interconnect.vip.vram.copy_from_slice(&state.interconnect.vip.vram);

    vb.interconnect.vip.display_state = match state.interconnect.vip.display_state {
        version1::DisplayStateState::Idle => DisplayState::Idle,
        version1::DisplayStateState::LeftFramebuffer => DisplayState::LeftFramebuffer,
        version1::DisplayStateState::RightFramebuffer => DisplayState::RightFramebuffer,
        version1::DisplayStateState::Finished => DisplayState::Finished,
    };

    vb.interconnect.vip.drawing_state = match state.interconnect.vip.drawing_state {
        version1::DrawingStateState::Idle => DrawingState::Idle,
        version1::DrawingStateState::Drawing => DrawingState::Drawing,
    };

    vb.interconnect.vip.reg_intpnd_lfbend = state.interconnect.vip.reg_intpnd_lfbend;
    vb.interconnect.vip.reg_intpnd_rfbend = state.interconnect.vip.reg_intpnd_rfbend;
    vb.interconnect.vip.reg_intpnd_gamestart = state.interconnect.vip.reg_intpnd_gamestart;
    vb.interconnect.vip.reg_intpnd_framestart = state.interconnect.vip.reg_intpnd_framestart;
    vb.interconnect.vip.reg_intpnd_sbhit = state.interconnect.vip.reg_intpnd_sbhit;
    vb.interconnect.vip.reg_intpnd_xpend = state.interconnect.vip.reg_intpnd_xpend;

    vb.interconnect.vip.reg_intenb_lfbend = state.interconnect.vip.reg_intenb_lfbend;
    vb.interconnect.vip.reg_intenb_rfbend = state.interconnect.vip.reg_intenb_rfbend;
    vb.interconnect.vip.reg_intenb_gamestart = state.interconnect.vip.reg_intenb_gamestart;
    vb.interconnect.vip.reg_intenb_framestart = state.interconnect.vip.reg_intenb_framestart;
    vb.interconnect.vip.reg_intenb_sbhit = state.interconnect.vip.reg_intenb_sbhit;
    vb.interconnect.vip.reg_intenb_xpend = state.interconnect.vip.reg_intenb_xpend;

    vb.interconnect.vip.reg_dpctrl_disp = state.interconnect.vip.reg_dpctrl_disp;
    vb.interconnect.vip.reg_dpctrl_synce = state.interconnect.vip.reg_dpctrl_synce;

    vb.interconnect.vip.reg_xpctrl_xpen = state.interconnect.vip.reg_xpctrl_xpen;
    vb.interconnect.vip.reg_xpctrl_sbcount = state.interconnect.vip.reg_xpctrl_sbcount;
    vb.interconnect.vip.reg_xpctrl_sbcmp = state.interconnect.vip.reg_xpctrl_sbcmp;
    vb.interconnect.vip.reg_xpctrl_sbout = state.interconnect.vip.reg_xpctrl_sbout;

    vb.interconnect.vip.reg_frmcyc = state.interconnect.vip.reg_frmcyc;

    vb.interconnect.vip.reg_brta = state.interconnect.vip.reg_brta;
    vb.interconnect.vip.reg_brtb = state.interconnect.vip.reg_brtb;
    vb.interconnect.vip.reg_brtc = state.interconnect.vip.reg_brtc;

    vb.interconnect.vip.reg_spt0 = state.interconnect.vip.reg_spt0;
    vb.interconnect.vip.reg_spt1 = state.interconnect.vip.reg_spt1;
    vb.interconnect.vip.reg_spt2 = state.interconnect.vip.reg_spt2;
    vb.interconnect.vip.reg_spt3 = state.interconnect.vip.reg_spt3;

    vb.interconnect.vip.reg_gplt0 = state.interconnect.vip.reg_gplt0;
    vb.interconnect.vip.reg_gplt1 = state.interconnect.vip.reg_gplt1;
    vb.interconnect.vip.reg_gplt2 = state.interconnect.vip.reg_gplt2;
    vb.interconnect.vip.reg_gplt3 = state.interconnect.vip.reg_gplt3;

    vb.interconnect.vip.reg_jplt0 = state.interconnect.vip.reg_jplt0;
    vb.interconnect.vip.reg_jplt1 = state.interconnect.vip.reg_jplt1;
    vb.interconnect.vip.reg_jplt2 = state.interconnect.vip.reg_jplt2;
    vb.interconnect.vip.reg_jplt3 = state.interconnect.vip.reg_jplt3;

    vb.interconnect.vip.reg_bkcol = state.interconnect.vip.reg_bkcol;

    vb.interconnect.vip.display_frame_eighth_clock_counter = state.interconnect.vip.display_frame_eighth_clock_counter;
    vb.interconnect.vip.display_frame_eighth_counter = state.interconnect.vip.display_frame_eighth_counter;

    vb.interconnect.vip.drawing_block_counter = state.interconnect.vip.drawing_block_counter;
    vb.interconnect.vip.drawing_sbout_counter = state.interconnect.vip.drawing_sbout_counter;

    vb.interconnect.vip.fclk = state.interconnect.vip.fclk;

    vb.interconnect.vip.display_first_framebuffers = state.interconnect.vip.display_first_framebuffers;
    vb.interconnect.vip.last_bkcol = state.interconnect.vip.last_bkcol;

    // Vsu
    vb.interconnect.vsu.waveform_data.copy_from_slice(&state.interconnect.vsu.waveform_data);
    vb.interconnect.vsu.mod_data.copy_from_slice(&state.interconnect.vsu.mod_data);

    apply_standard_sound_state(&mut vb.interconnect.vsu.sound1, &state.interconnect.vsu.sound1);
    apply_standard_sound_state(&mut vb.interconnect.vsu.sound2, &state.interconnect.vsu.sound2);
    apply_standard_sound_state(&mut vb.interconnect.vsu.sound3, &state.interconnect.vsu.sound3);
    apply_standard_sound_state(&mut vb.interconnect.vsu.sound4, &state.interconnect.vsu.sound4);

    apply_int_reg_state(&mut vb.interconnect.vsu.sound5.reg_int, &state.interconnect.vsu.sound5.reg_int);

    apply_lrv_reg_state(&mut vb.interconnect.vsu.sound5.reg_lrv, &state.interconnect.vsu.sound5.reg_lrv);

    vb.interconnect.vsu.sound5.fql = state.interconnect.vsu.sound5.fql;
    vb.interconnect.vsu.sound5.fqh = state.interconnect.vsu.sound5.fqh;
    vb.interconnect.vsu.sound5.frequency_low = state.interconnect.vsu.sound5.frequency_low;
    vb.interconnect.vsu.sound5.frequency_high = state.interconnect.vsu.sound5.frequency_high;
    vb.interconnect.vsu.sound5.next_frequency_low = state.interconnect.vsu.sound5.next_frequency_low;
    vb.interconnect.vsu.sound5.next_frequency_high = state.interconnect.vsu.sound5.next_frequency_high;

    apply_envelope_state(&mut vb.interconnect.vsu.sound5.envelope, &state.interconnect.vsu.sound5.envelope);

    vb.interconnect.vsu.sound5.reg_sweep_mod_enable = state.interconnect.vsu.sound5.reg_sweep_mod_enable;
    vb.interconnect.vsu.sound5.reg_mod_repeat = state.interconnect.vsu.sound5.reg_mod_repeat;
    vb.interconnect.vsu.sound5.reg_function = state.interconnect.vsu.sound5.reg_function;

    vb.interconnect.vsu.sound5.reg_sweep_mod_base_interval = state.interconnect.vsu.sound5.reg_sweep_mod_base_interval;
    vb.interconnect.vsu.sound5.reg_sweep_mod_interval = state.interconnect.vsu.sound5.reg_sweep_mod_interval;
    vb.interconnect.vsu.sound5.reg_sweep_direction = state.interconnect.vsu.sound5.reg_sweep_direction;
    vb.interconnect.vsu.sound5.reg_sweep_shift_amount = state.interconnect.vsu.sound5.reg_sweep_shift_amount;

    vb.interconnect.vsu.sound5.ram = state.interconnect.vsu.sound5.ram;

    vb.interconnect.vsu.sound5.frequency_counter = state.interconnect.vsu.sound5.frequency_counter;
    vb.interconnect.vsu.sound5.phase = state.interconnect.vsu.sound5.phase;

    vb.interconnect.vsu.sound5.sweep_mod_counter = state.interconnect.vsu.sound5.sweep_mod_counter;
    vb.interconnect.vsu.sound5.mod_phase = state.interconnect.vsu.sound5.mod_phase;

    apply_int_reg_state(&mut vb.interconnect.vsu.sound6.reg_int, &state.interconnect.vsu.sound6.reg_int);

    apply_lrv_reg_state(&mut vb.interconnect.vsu.sound6.reg_lrv, &state.interconnect.vsu.sound6.reg_lrv);

    vb.interconnect.vsu.sound6.fql = state.interconnect.vsu.sound6.fql;
    vb.interconnect.vsu.sound6.fqh = state.interconnect.vsu.sound6.fqh;

    apply_envelope_state(&mut vb.interconnect.vsu.sound6.envelope, &state.interconnect.vsu.sound6.envelope);

    vb.interconnect.vsu.sound6.reg_noise_control = state.interconnect.vsu.sound6.reg_noise_control;

    vb.interconnect.vsu.sound6.frequency_counter = state.interconnect.vsu.sound6.frequency_counter;
    vb.interconnect.vsu.sound6.shift = state.interconnect.vsu.sound6.shift;
    vb.interconnect.vsu.sound6.output = state.interconnect.vsu.sound6.output;

    vb.interconnect.vsu.duration_clock_counter = state.interconnect.vsu.duration_clock_counter;
    vb.interconnect.vsu.envelope_clock_counter = state.interconnect.vsu.envelope_clock_counter;
    vb.interconnect.vsu.frequency_clock_counter = state.interconnect.vsu.frequency_clock_counter;
    vb.interconnect.vsu.sweep_mod_clock_counter = state.interconnect.vsu.sweep_mod_clock_counter;
    vb.interconnect.vsu.noise_clock_counter = state.interconnect.vsu.noise_clock_counter;
    vb.interconnect.vsu.sample_clock_counter = state.interconnect.vsu.sample_clock_counter;

    // Timer
    vb.interconnect.timer.t_clk_sel = match state.interconnect.timer.t_clk_sel {
        version1::IntervalState::Large => Interval::Large,
        version1::IntervalState::Small => Interval::Small,
    };
    vb.interconnect.timer.tim_z_int = state.interconnect.timer.tim_z_int;
    vb.interconnect.timer.z_stat = state.interconnect.timer.z_stat;
    vb.interconnect.timer.t_enb = state.interconnect.timer.t_enb;
    vb.interconnect.timer.reload = state.interconnect.timer.reload;
    vb.interconnect.timer.counter = state.interconnect.timer.counter;

    vb.interconnect.timer.tick_counter = state.interconnect.timer.tick_counter;
    vb.interconnect.timer.zero_interrupt = state.interconnect.timer.zero_interrupt;

    // Game pad
    vb.interconnect.game_pad.a_pressed = state.interconnect.game_pad.a_pressed;
    vb.interconnect.game_pad.b_pressed = state.interconnect.game_pad.b_pressed;
    vb.interconnect.game_pad.start_pressed = state.interconnect.game_pad.start_pressed;
    vb.interconnect.game_pad.select_pressed = state.interconnect.game_pad.select_pressed;
    vb.interconnect.game_pad.l_pressed = state.interconnect.game_pad.l_pressed;
    vb.interconnect.game_pad.r_pressed = state.interconnect.game_pad.r_pressed;
    vb.interconnect.game_pad.left_d_pad_up_pressed = state.interconnect.game_pad.left_d_pad_up_pressed;
    vb.interconnect.game_pad.left_d_pad_down_pressed = state.interconnect.game_pad.left_d_pad_down_pressed;
    vb.interconnect.game_pad.left_d_pad_left_pressed = state.interconnect.game_pad.left_d_pad_left_pressed;
    vb.interconnect.game_pad.left_d_pad_right_pressed = state.interconnect.game_pad.left_d_pad_right_pressed;
    vb.interconnect.game_pad.right_d_pad_up_pressed = state.interconnect.game_pad.right_d_pad_up_pressed;
    vb.interconnect.game_pad.right_d_pad_down_pressed = state.interconnect.game_pad.right_d_pad_down_pressed;
    vb.interconnect.game_pad.right_d_pad_left_pressed = state.interconnect.game_pad.right_d_pad_left_pressed;
    vb.interconnect.game_pad.right_d_pad_right_pressed = state.interconnect.game_pad.right_d_pad_right_pressed;

    // Com port
    vb.interconnect.com_port.cdtr = state.interconnect.com_port.cdtr;
    vb.interconnect.com_port.cdrr = state.interconnect.com_port.cdrr;

    vb.interconnect.com_port.c_stat = state.interconnect.com_port.c_stat;

    vb.interconnect.com_port.transfer_bit_index = state.interconnect.com_port.transfer_bit_index;

    // Cpu
    vb.cpu.reg_pc = state.cpu.reg_pc;

    vb.cpu.reg_gpr.copy_from_slice(&state.cpu.reg_gpr[..]);

    vb.cpu.reg_eipc = state.cpu.reg_eipc;
    vb.cpu.reg_eipsw = state.cpu.reg_eipsw;
    vb.cpu.reg_ecr = state.cpu.reg_ecr;
    vb.cpu.reg_fepc = state.cpu.reg_fepc;
    vb.cpu.reg_fepsw = state.cpu.reg_fepsw;

    vb.cpu.psw_zero = state.cpu.psw_zero;
    vb.cpu.psw_sign = state.cpu.psw_sign;
    vb.cpu.psw_overflow = state.cpu.psw_overflow;
    vb.cpu.psw_carry = state.cpu.psw_carry;
    vb.cpu.psw_fp_precision_degredation = state.cpu.psw_fp_precision_degredation;
    vb.cpu.psw_fp_underflow = state.cpu.psw_fp_underflow;
    vb.cpu.psw_fp_overflow = state.cpu.psw_fp_overflow;
    vb.cpu.psw_fp_zero_division = state.cpu.psw_fp_zero_division;
    vb.cpu.psw_fp_invalid_operation = state.cpu.psw_fp_invalid_operation;
    vb.cpu.psw_fp_reserved_operand = state.cpu.psw_fp_reserved_operand;
    vb.cpu.psw_interrupt_disable = state.cpu.psw_interrupt_disable;
    vb.cpu.psw_address_trap_enable = state.cpu.psw_address_trap_enable;
    vb.cpu.psw_exception_pending = state.cpu.psw_exception_pending;
    vb.cpu.psw_nmi_pending = state.cpu.psw_nmi_pending;
    vb.cpu.psw_interrupt_mask_level = state.cpu.psw_interrupt_mask_level;

    vb.cpu.is_halted = state.cpu.is_halted;

    vb.cpu.cache.hits = state.cpu.cache.hits;
    vb.cpu.cache.misses = state.cpu.cache.misses;
    vb.cpu.cache.is_enabled = state.cpu.cache.is_enabled;
    for (entry, state) in vb.cpu.cache.entries.iter_mut().zip(state.cpu.cache.entries.iter()) {
        entry.tag = state.tag;
        entry.base_addr = state.base_addr;
        entry.subblock_valid.copy_from_slice(&state.subblock_valid);
    }

    vb.cpu.watchpoints = state.cpu.watchpoints.clone();
}

fn apply_standard_sound_state(sound: &mut StandardSound, state: &version1::StandardSoundState) {
    apply_int_reg_state(&mut sound.reg_int, &state.reg_int);

    apply_lrv_reg_state(&mut sound.reg_lrv, &state.reg_lrv);

    sound.fql = state.fql;
    sound.fqh = state.fqh;

    apply_envelope_state(&mut sound.envelope, &state.envelope);

    sound.ram = state.ram;

    sound.frequency_counter = state.frequency_counter;
    sound.phase = state.phase;
}

fn apply_int_reg_state(int_reg: &mut IntReg, state: &version1::IntRegState) {
    int_reg.output_enable = state.output_enable;
    int_reg.interval_data = state.interval_data;
    int_reg.interval_counter_setting_values = state.interval_counter_setting_values;

    int_reg.interval_counter = state.interval_counter;
}

fn apply_lrv_reg_state(lrv_reg: &mut LrvReg, state: &version1::LrvRegState) {
    lrv_reg.left = state.left;
    lrv_reg.right = state.right;
}

fn apply_envelope_state(envelope: &mut Envelope, state: &version1::EnvelopeState) {
    envelope.reg_data_reload = state.reg_data_reload;
    envelope.reg_data_direction = state.reg_data_direction;
    envelope.reg_data_step_interval = state.reg_data_step_interval;

    envelope.reg_control_repeat = state.reg_control_repeat;
    envelope.reg_control_enable = state.reg_control_enable;

    envelope.level = state.level;

    envelope.envelope_counter = state.envelope_counter;
}
