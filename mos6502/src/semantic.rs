/*
 * Panopticon - A libre disassembler
 * Copyright (C) 2015  Panopticon authors
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use disassembler::*;
use panopticon_core::{Guard, Lvalue, Result, Rvalue, State, Statement};
use std::convert::Into;

pub fn nop(_: &mut Variant) -> Result<Vec<Statement>> {
    Ok(vec![])
}

pub fn nop_r(_: &mut Variant, _: Rvalue) -> Result<Vec<Statement>> {
    Ok(vec![])
}

pub fn adc(_cg: &mut Variant, r: Rvalue) -> Result<Vec<Statement>> {
    rreil!{
        zext/8 carry:8, C:1;
        add res:8, A:8, (r);
        add res:8, res:8, carry:8;

        cmpeq Z:1, [0]:8, res:8;

        cmpleu c1:1, res:8, A:8;
        cmpeq c2:1, res:8, A:8;
        and c2:1, c2:1, C:1;
        and C:1, c1:1, c2:1;

        cmples N:1, res:8, [0]:8;

        cmples v1:1, res:8, A:8;
        cmpeq v2:1, res:8, A:8;
        and v2:1, v2:1, C:1;
        and V:1, v1:1, v2:1;

        mov A:8, res:8;
    }
    /*
    // This will contain our result.  Bit 8 is carry.
    let result = new_temp(16);
    let result_c6 = new_temp(8);
    let result_n = new_temp(8);

    // Decimal mode is a flag.  So we have to calculate both values and _select the right one.
    let normal = new_temp(16);
    let normal_c6 = new_temp(8);
    let normal_n = new_temp(8);
    let decimal = new_temp(16);
    let decimal_c6 = new_temp(8);
    let decimal_n = new_temp(8);

    // These two are used for c6 calculations (overflow).
    // V = C6 xor C7 (carry).  We get C6 by blanking out the top bit.
    let _v1 = new_temp(8);
    let _v2 = new_temp(8);

    // Normal mode.
    _cg.assign(&normal, &*A);
    _cg.add_i(&normal, &normal.to_rv(), &r);
    _cg.add_i(&normal, &normal.to_rv(), &C.to_rv());
    _cg.rshift_i(&normal_n, &normal.to_rv(), &7);
    _cg.and_i(&normal_n, &normal_n.to_rv(), &1);

    _cg.and_i(&_v1, &A.to_rv(), &0x7f);
    _cg.and_i(&_v2, &r, &0x7f);
    _cg.add_i(&_v1, &_v1.to_rv(), &_v2.to_rv());
    _cg.add_i(&_v1, &_v1.to_rv(), &C.to_rv());
    _cg.rshift_i(&normal_c6, &_v1.to_rv(), &7);


    // Decimal mode.  It's complicated: http://www.6502.org/tutorials/decimal_mode.html

    // 1a. Decimal
    let al = new_temp(8);
    _cg.assign(&al, &*A);
    _cg.and_i(&al, &al.to_rv(), &0xf);

    let lo = new_temp(8);
    _cg.assign(&lo, &r);
    _cg.and_i(&lo, &lo.to_rv(), &0xf);

    _cg.add_i(&al, &al.to_rv(), &lo);
    _cg.add_i(&al, &al.to_rv(), &C.to_rv());

    // 1b. We have now al = (A & $0F) + (R & $0F) + C <= 0x1f and have to compare to >= 0x0a.
    let adjust = new_temp(8);
    _cg.add_i(&adjust, &al.to_rv(), &0xe6);        // -a in 2-complement
    _cg.rshift_i(&adjust, &adjust.to_rv(), &7);    // N bit means >= 0x0a

    let adjusted = new_temp(8);
    _cg.assign(&adjusted, &al.to_rv());
    _cg.add_i(&adjusted, &adjusted.to_rv(), &6);
    _cg.and_i(&adjusted, &adjusted.to_rv(), &0xf);
    _cg.or_i(&adjusted, &adjusted.to_rv(), &0x10);

    _select(_cg, &lo, &al.to_rv(), &adjusted.to_rv(), &adjust.to_rv());
    _cg.assign(&al, &lo.to_rv());

    // 1c.
    let _decimal = new_temp(16);
    _cg.and_i(&_decimal, &A.to_rv(), &0xf0);
    _cg.add_i(&_decimal, &_decimal.to_rv(), &r);
    _cg.and_i(&_decimal, &_decimal.to_rv(), &0x1f0);
    _cg.add_i(&_decimal, &_decimal.to_rv(), &al.to_rv());

    // In decimal mode, the negative flag is the 8th bit of the previous addition (1c).
    _cg.rshift_i(&decimal_n, &_decimal.to_rv(), &7);
    _cg.and_i(&decimal_n, &decimal_n.to_rv(), &1);

    // In decimal mode, the overflow flag is the C6 of the previous addition (1c).
    _cg.and_i(&_v1, &A.to_rv(), &0x70);
    _cg.and_i(&_v2, &r, &0x70);
    _cg.add_i(&_v1, &_v1.to_rv(), &_v2.to_rv());
    _cg.add_i(&_v1, &_v1.to_rv(), &al.to_rv());
    _cg.rshift_i(&decimal_c6, &_v1.to_rv(), &7);

    // 1e. Compare to 0xa0.  Note that _decimal is max. 0x1ff (because al is max. 0x1f)
    let hiadjust = new_temp(16);
    _cg.add_i(&hiadjust, &_decimal.to_rv(), &0xfe60);  // -a0 in 2-complement
    _cg.rshift_i(&hiadjust, &hiadjust.to_rv(), &15);   // N bit means > 0xa0.  This is also the new carry!

    let hiadjusted = new_temp(16);
    _cg.assign(&adjusted, &_decimal.to_rv());
    _cg.add_i(&hiadjusted, &hiadjusted.to_rv(), &0x60);
    _cg.and_i(&hiadjusted, &hiadjusted.to_rv(), &0xff);
    _cg.or_i(&hiadjusted, &hiadjusted.to_rv(), &0x100); // Set new carry.

    _select(_cg, &decimal, &_decimal.to_rv(), &hiadjusted.to_rv(), &hiadjust.to_rv());

    // Finally, select the result that is actually used.
    _select(_cg, &result, &normal.to_rv(), &decimal.to_rv(), &D.to_rv());
    _select(_cg, &result_c6, &normal_c6.to_rv(), &decimal_c6.to_rv(), &D.to_rv());
    _select(_cg, &result_n, &normal_n.to_rv(), &decimal_n.to_rv(), &D.to_rv());

    // Output all results.
    _cg.assign(&*A, &result.to_rv());
    _cg.rshift_i(&*C, &result.to_rv(), &8);
    _cg.assign(&*N, &result_n.to_rv());
    _cg.xor_i(&*V, &result_c6.to_rv(), &C.to_rv());
    _cg.equal_i(&*Z, &A.to_rv(), &0);*/
}

pub fn and(_cg: &mut Variant, r: Rvalue) -> Result<Vec<Statement>> {
    rreil!{
        and A:8, A:8, (r);
        cmpeq Z:1, A:8, [0]:8;
        cmples N:1, A:8, [0]:8;
    }
}

pub fn asl(_cg: &mut Variant, _r: Rvalue) -> Result<Vec<Statement>> {
    rreil!{
        mov C:1, A:1/7;
        shl A:8, A:8, [1]:8;
        cmpeq Z:1, A:8, [0]:8;
        cmples N:1, A:8, [0]:8;
    }
}

pub fn bit(_cg: &mut Variant, r: Rvalue) -> Result<Vec<Statement>> {
    rreil!{
        and res:8, A:8, (r);
        cmpeq Z:1, res:8, [0]:8;
        cmples N:1, res:8, [0]:8;
        mov V:1, res:1/7;
    }
}


pub fn brk(_: &mut Variant) -> Result<Vec<Statement>> {
    /* Well.  We could simulate BRK up to the indirect jump at the NMI vector.
       So we add the code to do that here.  But without the ROM, this is useless
       (and with user-provided NMI handlers it would be very dynamic).
       For now, it seems simpler to just ignore the BRK instruction and all its
       side effects.  */
    /*
       let reg = new_temp(8);
       _cg.assign(&reg, &PC.to_rvalue());
       _push(_cg, &reg.to_rv());
       _cg.rshift_i(&pc, &PC.to_rvalue(), &8);
       _push(_cg, &reg.to_rv());
       _pushf(_cg, &0);
       */
    Ok(vec![])
}

pub fn clc(_cg: &mut Variant) -> Result<Vec<Statement>> {
    rreil!{
        mov C:1, [0]:1;
    }
}

pub fn cli(_cg: &mut Variant) -> Result<Vec<Statement>> {
    rreil!{
        mov I:1, [0]:1;
    }
}

pub fn cld(_cg: &mut Variant) -> Result<Vec<Statement>> {
    rreil!{
        mov D:1, [0]:1;
    }
}

pub fn sec(_cg: &mut Variant) -> Result<Vec<Statement>> {
    rreil!{
        mov C:1, [1]:1;
    }
}

pub fn sei(_cg: &mut Variant) -> Result<Vec<Statement>> {
    rreil!{
        mov I:1, [1]:1;
    }
}

pub fn clv(_cg: &mut Variant) -> Result<Vec<Statement>> {
    rreil!{
        mov V:1, [0]:1;
    }
}

pub fn sed(_cg: &mut Variant) -> Result<Vec<Statement>> {
    rreil!{
        mov D:1, [1]:1;
    }
}

fn cmp(_cg: &mut Variant, r1: Rvalue, r2: Rvalue) -> Result<Vec<Statement>> {
    rreil!{
        cmpltu C:1, (r1), (r2);
        mov N:1, C:1;
        cmpeq Z:1, (r1), (r2);
    }
}

pub fn cpx(_cg: &mut Variant, r: Rvalue) -> Result<Vec<Statement>> {
    cmp(_cg, rreil_rvalue!{ X:8 }, r)
}

pub fn cpy(_cg: &mut Variant, r: Rvalue) -> Result<Vec<Statement>> {
    cmp(_cg, rreil_rvalue!{ Y:8 }, r)
}

pub fn cpa(_cg: &mut Variant, r: Rvalue) -> Result<Vec<Statement>> {
    cmp(_cg, rreil_rvalue!{ A:8 }, r)
}

fn dec(_cg: &mut Variant, l: Lvalue, r: Rvalue) -> Result<Vec<Statement>> {
    rreil!{
        sub (l), (r), [1]:8;
        cmpeq Z:1, (l), [0]:8;
        cmplts N:1, (r), [0]:8;
    }
}

pub fn dea(_cg: &mut Variant, r: Rvalue) -> Result<Vec<Statement>> {
    dec(_cg, rreil_lvalue!{ A:8 }, r)
}

pub fn dex(_cg: &mut Variant) -> Result<Vec<Statement>> {
    dec(_cg, rreil_lvalue!{ X:8 }, rreil_rvalue!{ X:8 })
}

pub fn dey(_cg: &mut Variant) -> Result<Vec<Statement>> {
    dec(_cg, rreil_lvalue!{ Y:8 }, rreil_rvalue!{ Y:8 })
}

pub fn eor(_cg: &mut Variant, r: Rvalue) -> Result<Vec<Statement>> {
    rreil!{
        xor A:8, (r), A:8;
        cmpeq Z:1, A:8, [0]:8;
        cmplts N:1, A:8, [0]:8;
    }
}

fn inc(_cg: &mut Variant, l: Lvalue, r: Rvalue) -> Result<Vec<Statement>> {
    rreil!{
        add (l), (r), [1]:8;
        cmpeq Z:1, (l), [0]:8;
        cmplts N:1, (l), [0]:8;
    }
}

pub fn ina(_cg: &mut Variant, r: Rvalue) -> Result<Vec<Statement>> {
    inc(_cg, rreil_lvalue!{ A:8 }, r)
}

pub fn inx(_cg: &mut Variant) -> Result<Vec<Statement>> {
    inc(_cg, rreil_lvalue!{ X:8 }, rreil_rvalue!{ X:8 })
}

pub fn iny(_cg: &mut Variant) -> Result<Vec<Statement>> {
    inc(_cg, rreil_lvalue!{ Y:8 }, rreil_rvalue!{ Y:8 })
}

fn ld(_cg: &mut Variant, l: Lvalue, r: Rvalue) -> Result<Vec<Statement>> {
    rreil!{
        mov (l), (r);
        cmpeq Z:1, (l), [0]:8;
        cmplts N:1, (l), [0]:8;
    }
}

pub fn lda(_cg: &mut Variant, r: Rvalue) -> Result<Vec<Statement>> {
    ld(_cg, rreil_lvalue!{ A:8 }, r)
}

pub fn ldx(_cg: &mut Variant, r: Rvalue) -> Result<Vec<Statement>> {
    ld(_cg, rreil_lvalue!{ X:8 }, r)
}

pub fn ldy(_cg: &mut Variant, r: Rvalue) -> Result<Vec<Statement>> {
    ld(_cg, rreil_lvalue!{ Y:8 }, r)
}

pub fn lsr(_cg: &mut Variant, r: Rvalue) -> Result<Vec<Statement>> {
    rreil!{
        mov C:1, A:1;
        shl A:8, A:8, (r);
        cmpeq Z:1, A:8, [0]:8;
        mov N:1, [0]:1;
    }
}

pub fn ora(_cg: &mut Variant, r: Rvalue) -> Result<Vec<Statement>> {
    rreil!{
        or A:8, (r), A:8;
        cmpeq Z:1, A:8, [0]:8;
        cmplts N:1, A:8, [0]:8;
    }
}

pub fn pha(_cg: &mut Variant) -> Result<Vec<Statement>> {
    rreil!{
        zext/9 sp:9, S:8;
        add sp:9, sp:9, [0x100]:9;

        store/ram sp:9, A:8;

        add sp:9, sp:9, [1]:9;
        mov S:8, sp:8;
    }
}

pub fn php(_cg: &mut Variant) -> Result<Vec<Statement>> {
    rreil!{
        zext/9 sp:9, S:8;
        add sp:9, sp:9, [0x100]:9;

        zext/8 flags:8, C:1;
        sel/1 flags:8, Z:1;
        sel/2 flags:8, I:1;
        sel/3 flags:8, D:1;
        sel/4 flags:8, B:1;
        sel/5 flags:8, ?;
        sel/6 flags:8, V:1;
        sel/7 flags:8, N:1;

        store/ram sp:9, flags:8;
        add sp:9, sp:9, [1]:9;
        mov S:8, sp:8;
    }
}

pub fn pla(_cg: &mut Variant) -> Result<Vec<Statement>> {
    rreil!{
        zext/9 sp:9, S:8;
        add sp:9, sp:9, [0x100]:9;

        add sp:9, sp:9, [1]:9;
        load/ram A:8, sp:9;

        mov S:8, sp:8;

        cmpeq Z:1, A:8, [0]:8;
        cmplts N:1, A:8, [0]:8;
    }
}

pub fn plp(_cg: &mut Variant) -> Result<Vec<Statement>> {
    rreil!{
        zext/9 sp:9, S:8;
        add sp:9, sp:9, [0x100]:9;

        add sp:9, sp:9, [1]:9;
        load/ram flags:8, sp:9;

        mov C:1, flags:1;
        mov Z:1, flags:1/1;
        mov I:1, flags:1/2;
        mov D:1, flags:1/3;
        mov V:1, flags:1/6;
        mov N:1, flags:1/7;

        mov S:8, sp:8;

        cmpeq Z:1, A:8, [0]:8;
        cmplts N:1, A:8, [0]:8;
    }
}


pub fn rol(_cg: &mut Variant, _r: Rvalue) -> Result<Vec<Statement>> {
    let r = Lvalue::from_rvalue(_r).unwrap();
    rreil!{
        mov hb:1, (r.extract(1,7).unwrap());
        shl (r), (r), [1]:8;
        sel/7 (r), C:1;
        mov C:1, hb:1;
        cmpeq Z:1, (r), [0]:8;
        cmples N:1, (r), [0]:8;
    }
}

pub fn ror(_cg: &mut Variant, _r: Rvalue) -> Result<Vec<Statement>> {
    let r = Lvalue::from_rvalue(_r).unwrap();
    rreil!{
        mov lb:1, (r.extract(1,0).unwrap());
        shr (r), (r), [1]:8;
        sel/7 (r), C:1;
        mov C:1, lb:1;
        cmpeq Z:1, (r), [0]:8;
        cmples N:1, (r), [0]:8;
    }
}

/*pub fn rts(_: &mut Variant) -> Result<Vec<Statement>> {
    /* FIXME: Pop PC-1 from stack (so that the next instruction is fetched
       from TOS+1 */
    Ok(vec![])
}*/


pub fn sbc(_cg: &mut Variant, r: Rvalue) -> Result<Vec<Statement>> {
    rreil!{
        zext/8 carry:8, C:1;
        sub res:8, A:8, (r);
        add res:8, res:8, carry:8;

        cmpeq Z:1, [0]:8, res:8;

        cmpleu c1:1, res:8, A:8;
        cmpeq c2:1, res:8, A:8;
        and c2:1, c2:1, C:1;
        and C:1, c1:1, c2:1;

        cmples N:1, res:8, [0]:8;

        cmples v1:1, res:8, A:8;
        cmpeq v2:1, res:8, A:8;
        and v2:1, v2:1, C:1;
        and V:1, v1:1, v2:1;

        mov A:8, res:8;
    }
    /*
    // This will contain our result.  Bit 8 is carry.
    let result = new_temp(16);
    let result_c = new_temp(8);
    let result_v = new_temp(8);
    let result_n = new_temp(8);

    // Decimal mode is a flag.  So we have to calculate both values and _select the right one.
    let normal = new_temp(16);
    let _addend = new_temp(8);
    let decimal = new_temp(16);

    // These two are used for c6 calculations (overflow).
    // V = C6 xor C7 (carry).  We get C6 by blanking out the top bit.
    let _v1 = new_temp(8);
    let _v2 = new_temp(8);

    // Normal mode.  Same as adding 255-r.
    _cg.assign(&normal, &*A);
    _cg.xor_i(&_addend, &r, &0xff);
    _cg.add_i(&normal, &normal.to_rv(), &_addend.to_rv());
    _cg.add_i(&normal, &normal.to_rv(), &C.to_rv());

    // Common results.
    _cg.rshift_i(&result_c, &normal.to_rv(), &8);
    _cg.rshift_i(&result_n, &normal.to_rv(), &7);

    _cg.and_i(&_v1, &A.to_rv(), &0x7f);
    _cg.and_i(&_v2, &_addend.to_rv(), &0x7f);
    _cg.add_i(&_v1, &_v1.to_rv(), &_v2.to_rv());
    _cg.add_i(&_v1, &_v1.to_rv(), &C.to_rv());
    _cg.rshift_i(&result_v, &_v1.to_rv(), &7);
    _cg.xor_i(&result_v, &result_v.to_rv(), &result_c.to_rv());

    // Decimal mode.  It's complicated: http://www.6502.org/tutorials/decimal_mode.html

    // FIXME

    // 1a. Decimal
    let al = new_temp(8);
    _cg.assign(&al, &*A);
    _cg.and_i(&al, &al.to_rv(), &0xf);

    let lo = new_temp(8);
    _cg.assign(&lo, &r);
    _cg.and_i(&lo, &lo.to_rv(), &0xf);

    _cg.sub_i(&al, &al.to_rv(), &lo);
    _cg.add_i(&al, &al.to_rv(), &C.to_rv());
    _cg.sub_i(&al, &al.to_rv(), &1);

    // 1b. We have now al = (A & $0F) - (R & $0F) - 1 + C and have to compare to < 0.
    let adjust = new_temp(8);
    _cg.rshift_i(&adjust, &al.to_rv(), &7);    // N bit means < 0

    let adjusted = new_temp(8);
    _cg.assign(&adjusted, &al.to_rv());
    _cg.sub_i(&adjusted, &adjusted.to_rv(), &6);
    _cg.and_i(&adjusted, &adjusted.to_rv(), &0xf);
    _cg.sub_i(&adjusted, &adjusted.to_rv(), &0x10);

    _select(_cg, &lo, &al.to_rv(), &adjusted.to_rv(), &adjust.to_rv());
    _cg.assign(&al, &lo.to_rv());

    // 1c.
    let _decimal = new_temp(16);
    _cg.and_i(&_decimal, &A.to_rv(), &0xf0);
    _cg.sub_i(&_decimal, &_decimal.to_rv(), &r);
    _cg.add_i(&_decimal, &_decimal.to_rv(), &0x10); // Or sub r&0xf0 instead.
    _cg.and_i(&_decimal, &_decimal.to_rv(), &0xfff0);
    _cg.add_i(&_decimal, &_decimal.to_rv(), &al.to_rv());

    // 1e. Compare to 0.
    let hiadjust = new_temp(16);
    _cg.rshift_i(&hiadjust, &hiadjust.to_rv(), &15); // N bit means > 0xa0.

    let hiadjusted = new_temp(16);
    _cg.assign(&adjusted, &_decimal.to_rv());
    _cg.sub_i(&hiadjusted, &hiadjusted.to_rv(), &0x60);
    _select(_cg, &decimal, &_decimal.to_rv(), &hiadjusted.to_rv(), &hiadjust.to_rv());

    // Finally, select the result that is actually used.
    _select(_cg, &result, &normal.to_rv(), &decimal.to_rv(), &D.to_rv());

    // Output all results.
    _cg.assign(&*A, &result.to_rv());
    _cg.assign(&*C, &result_c.to_rv());
    _cg.assign(&*V, &result_v.to_rv());
    _cg.assign(&*N, &result_n.to_rv());
    _cg.equal_i(&*Z, &A.to_rv(), &0);*/
}

fn st(_cg: &mut Variant, reg: Lvalue, ptr: Rvalue) -> Result<Vec<Statement>> {
    rreil!{
        store/ram (reg), (ptr);
    }
}

pub fn sta(_cg: &mut Variant, r: Rvalue) -> Result<Vec<Statement>> {
    st(_cg, rreil_lvalue!{ A:8 }, r)
}

pub fn stx(_cg: &mut Variant, r: Rvalue) -> Result<Vec<Statement>> {
    st(_cg, rreil_lvalue!{ X:8 }, r)
}

pub fn sty(_cg: &mut Variant, r: Rvalue) -> Result<Vec<Statement>> {
    st(_cg, rreil_lvalue!{ Y:8 }, r)
}

pub fn trr(_cg: &mut Variant, src: &Lvalue, dst: &Lvalue) -> Result<Vec<Statement>> {
    rreil!{
        mov (dst), (src);
        cmpeq Z:1, (dst), [0]:8;
        cmplts N:1, (dst), [0]:8;
    }
}

pub fn tax(_cg: &mut Variant) -> Result<Vec<Statement>> {
    trr(_cg, &A, &X)
}

pub fn tay(_cg: &mut Variant) -> Result<Vec<Statement>> {
    trr(_cg, &A, &Y)
}

pub fn tsx(_cg: &mut Variant) -> Result<Vec<Statement>> {
    trr(_cg, &SP, &X)
}

pub fn txa(_cg: &mut Variant) -> Result<Vec<Statement>> {
    trr(_cg, &X, &A)
}

pub fn txs(_cg: &mut Variant) -> Result<Vec<Statement>> {
    trr(_cg, &X, &SP)
}

pub fn tya(_cg: &mut Variant) -> Result<Vec<Statement>> {
    trr(_cg, &Y, &A)
}

pub fn jmp_direct(st: &mut State<Mos>) -> bool {
    let next = Rvalue::new_u16(st.get_group("immlo") as u16 | ((st.get_group("immhi") as u16) << 8));

    st.mnemonic(
            3,
            "jmp",
            "{c:ram}",
            vec![next.clone()],
            &|_: &mut Variant| -> Result<Vec<Statement>> { Ok(vec![]) },
        )
        .unwrap();
    st.jump(next, Guard::always()).unwrap();

    true
}

pub fn jmp_indirect(st: &mut State<Mos>) -> bool {
    let ptr = Rvalue::new_u16(st.get_group("immlo") as u16 | ((st.get_group("immhi") as u16) << 8));

    st.mnemonic(
            0,
            "__fetch",
            "",
            vec![],
            &|_cg: &mut Variant| -> Result<Vec<Statement>> {
                rreil!{
            load/ram res:16, (ptr);
        }
            },
        )
        .unwrap();

    let next = rreil_rvalue!{ res:16 };

    st.mnemonic(
            3,
            "jmp",
            "{p:ram}",
            vec![ptr.clone()],
            &|_: &mut Variant| -> Result<Vec<Statement>> { Ok(vec![]) },
        )
        .unwrap();
    st.jump(next, Guard::always()).unwrap();

    true
}

pub fn jsr(st: &mut State<Mos>) -> bool {
    let next = Rvalue::new_u16(st.address as u16 + 3);
    let target = Rvalue::new_u16(st.get_group("immlo") as u16 | ((st.get_group("immhi") as u16) << 8));

    st.mnemonic(
            3,
            "jsr",
            "{c:ram}",
            vec![target.clone()],
            &|_cg: &mut Variant| -> Result<Vec<Statement>> {
                rreil!{
            call ?, (target);
        }
            },
        )
        .unwrap();
    st.jump(next, Guard::always()).unwrap();
    true
}
