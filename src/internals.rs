
use self::Class::*;
use self::State::*;

const ___: Class = Class::Invalid;
const __: State = State::Invalid;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum Class {
    CSpace, // space
    CWhite, // other whitespace
    CLcurb, // {
    CRcurb, // }
    CLsqrb, // [
    CRsqrb, // ]
    CColon, // :
    CComma, // ,
    CQuote, // "
    CBacks, // \
    CSlash, // /
    CPlus,  // +
    CMinus, // -
    CPoint, // .
    CZero , // 0
    CDigit, // 123456789
    CLowA,  // a
    CLowB,  // b
    CLowC,  // c
    CLowD,  // d
    CLowE,  // e
    CLowF,  // f
    CLowL,  // l
    CLowN,  // n
    CLowR,  // r
    CLowS,  // s
    CLowT,  // t
    CLowU,  // u
    CAbcdf, // ABCDF
    CE,     // E
    CEtc,   // everything else
    Invalid,
}

/// This array maps the 128 ASCII characters into character classes.
/// The remaining Unicode characters should be mapped to C_ETC.
/// Non-whitespace control characters are errors.
pub const ASCII_CLASS: [Class; 128] = [
    ___,    ___,    ___,    ___,    ___,    ___,      ___,  ___,
    ___,    CWhite, CWhite, ___,    ___,    CWhite,   ___,  ___,
    ___,    ___,    ___,    ___,    ___,    ___,      ___,  ___,
    ___,    ___,    ___,    ___,    ___,    ___,      ___,  ___,

    CSpace, CEtc,   CQuote, CEtc,   CEtc,   CEtc,   CEtc,   CEtc,
    CEtc,   CEtc,   CEtc,   CPlus,  CComma, CMinus, CPoint, CSlash,
    CZero,  CDigit, CDigit, CDigit, CDigit, CDigit, CDigit, CDigit,
    CDigit, CDigit, CColon, CEtc,   CEtc,   CEtc,   CEtc,   CEtc,

    CEtc,   CAbcdf, CAbcdf, CAbcdf, CAbcdf, CE,     CAbcdf, CEtc,
    CEtc,   CEtc,   CEtc,   CEtc,   CEtc,   CEtc,   CEtc,   CEtc,
    CEtc,   CEtc,   CEtc,   CEtc,   CEtc,   CEtc,   CEtc,   CEtc,
    CEtc,   CEtc,   CEtc,   CLsqrb, CBacks, CRsqrb, CEtc,   CEtc,

    CEtc,   CLowA,  CLowB,  CLowC,  CLowD,  CLowE,  CLowF,  CEtc,
    CEtc,   CEtc,   CEtc,   CEtc,   CLowL,  CEtc,   CLowN,  CEtc,
    CEtc,   CEtc,   CLowR,  CLowS,  CLowT,  CLowU,  CEtc,   CEtc,
    CEtc,   CEtc,   CEtc,   CLcurb, CEtc,   CRcurb, CEtc,   CEtc
];

/// The state codes.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum State {
    Go, // start
    Ok, // ok
    Ob, // object
    Ke, // key
    Co, // colon
    Va, // value
    Ar, // array
    St, // string
    Es, // escape
    U1, // u1
    U2, // u2
    U3, // u3
    U4, // u4
    Mi, // minus
    Ze, // zero
    In, // integer
    Fr, // fraction
    Fs, // fraction
    E1, // e
    E2, // ex
    E3, // exp
    T1, // tr
    T2, // tru
    T3, // true
    F1, // fa
    F2, // fal
    F3, // fals
    F4, // false
    N1, // nu
    N2, // nul
    N3, // null
    Wcl, // Wrong Colon : (-2)
    Wcm, // Wrong Comma , (-3)
    Wq,  // Wrong Quote " (-4)
    Wos, // Wrong Opening Squared [ (-5)
    Woc, // Wrong Opening Curly { (-6)
    Ws,  // Wrong Squared ] (-7)
    Wcu, // Wrong Curly } (-8)
    Wec, // Wrong Empty curly } (-9)
    Invalid,
}

impl State {
    pub fn is_valid(self) -> bool {
        match self {
            Wcl | Wcm | Wq | Wos | Woc | Ws | Wec | Wcu | State::Invalid => false,
            _ => true,
        }
    }
}

// Number of states by number of classes
pub const STATE_TRANSITION_TABLE: [[State; 31]; 31] = [
/*
    The state transition table takes the current state and the current symbol,
    and returns either a new state or an action. An action is represented as a
    negative number. A JSON text is accepted if at the end of the text the
    state is OK and if the mode is MODE_DONE.

                 white                                      1-9                                   ABCDF  etc
             space |  {  }  [  ]  :  ,  "  \  /  +  -  .  0  |  a  b  c  d  e  f  l  n  r  s  t  u  |  E  |*/
/*start  GO*/ [Go, Go,Woc, __,Wos, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __],
/*ok     OK*/ [Ok, Ok, __,Wcu, __, Ws, __, Wcm,__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __],
/*object OB*/ [Ob, Ob, __,Wec, __, __, __, __, St, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __],
/*key    KE*/ [Ke, Ke, __, __, __, __, __, __, St, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __],
/*colon  CO*/ [Co, Co, __, __, __, __,Wcl, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __],
/*value  VA*/ [Va, Va,Woc, __,Wos, __, __, __, St, __, __, __, Mi, __, Ze, In, __, __, __, __, __, F1, __, N1, __, __, T1, __, __, __, __],
/*array  AR*/ [Ar, Ar,Woc, __,Wos, Ws, __, __, St, __, __, __, Mi, __, Ze, In, __, __, __, __, __, F1, __, N1, __, __, T1, __, __, __, __],
/*string ST*/ [St, __, St, St, St, St, St, St, Wq, Es, St, St, St, St, St, St, St, St, St, St, St, St, St, St, St, St, St, St, St, St, St],
/*escape ES*/ [__, __, __, __, __, __, __, __, St, St, St, __, __, __, __, __, __, St, __, __, __, St, __, St, St, __, St, U1, __, __, __],
/*u1     U1*/ [__, __, __, __, __, __, __, __, __, __, __, __, __, __, U2, U2, U2, U2, U2, U2, U2, U2, __, __, __, __, __, __, U2, U2, __],
/*u2     U2*/ [__, __, __, __, __, __, __, __, __, __, __, __, __, __, U3, U3, U3, U3, U3, U3, U3, U3, __, __, __, __, __, __, U3, U3, __],
/*u3     U3*/ [__, __, __, __, __, __, __, __, __, __, __, __, __, __, U4, U4, U4, U4, U4, U4, U4, U4, __, __, __, __, __, __, U4, U4, __],
/*u4     U4*/ [__, __, __, __, __, __, __, __, __, __, __, __, __, __, St, St, St, St, St, St, St, St, __, __, __, __, __, __, St, St, __],
/*minus  MI*/ [__, __, __, __, __, __, __, __, __, __, __, __, __, __, Ze, In, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __],
/*zero   ZE*/ [Ok, Ok, __,Wcu, __, Ws, __,Wcm, __, __, __, __, __, Fr, __, __, __, __, __, __, E1, __, __, __, __, __, __, __, __, E1, __],
/*int    IN*/ [Ok, Ok, __,Wcu, __, Ws, __,Wcm, __, __, __, __, __, Fr, In, In, __, __, __, __, E1, __, __, __, __, __, __, __, __, E1, __],
/*frac   FR*/ [__, __, __, __, __, __, __, __, __, __, __, __, __, __, Fs, Fs, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __],
/*fracs  FS*/ [Ok, Ok, __,Wcu, __, Ws, __,Wcm, __, __, __, __, __, __, Fs, Fs, __, __, __, __, E1, __, __, __, __, __, __, __, __, E1, __],
/*e      E1*/ [__, __, __, __, __, __, __, __, __, __, __, E2, E2, __, E3, E3, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __],
/*ex     E2*/ [__, __, __, __, __, __, __, __, __, __, __, __, __, __, E3, E3, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __],
/*exp    E3*/ [Ok, Ok, __,Wcu, __, Ws, __,Wcm, __, __, __, __, __, __, E3, E3, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __],
/*tr     T1*/ [__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, T2, __, __, __, __, __, __],
/*tru    T2*/ [__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, T3, __, __, __],
/*true   T3*/ [__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, Ok, __, __, __, __, __, __, __, __, __, __],
/*fa     F1*/ [__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, F2, __, __, __, __, __, __, __, __, __, __, __, __, __, __],
/*fal    F2*/ [__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, F3, __, __, __, __, __, __, __, __],
/*fals   F3*/ [__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, F4, __, __, __, __, __],
/*false  F4*/ [__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, Ok, __, __, __, __, __, __, __, __, __, __],
/*nu     N1*/ [__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, N2, __, __, __],
/*nul    N2*/ [__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, N3, __, __, __, __, __, __, __, __],
/*null   N3*/ [__, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, Ok, __, __, __, __, __, __, __, __],
];

/// These modes can be pushed on the stack.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    Array,
    Done,
    Key,
    Object,
}
