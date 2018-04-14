// auto-generated: "lalrpop 0.14.0"
use std::str::FromStr;
use gosgf::*;
use std::collections::HashMap;
use regex;
#[allow(unused_extern_crates)]
extern crate lalrpop_util as __lalrpop_util;

mod __parse__Collection {
    #![allow(non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens)]

    use std::str::FromStr;
    use gosgf::*;
    use std::collections::HashMap;
    use regex;
    #[allow(unused_extern_crates)]
    extern crate lalrpop_util as __lalrpop_util;
    use super::__intern_token::Token;
    #[allow(dead_code)]
    pub enum __Symbol<'input>
     {
        Termr_23_22_3b_22_23(&'input str),
        Termr_23_22_5bA_2dZ_5d_2b_5c_5c_5b_5b_5e_5c_5c_5d_5d_2a_5c_5c_5d_22_23(&'input str),
        Termr_23_22_5c_5c_28_22_23(&'input str),
        Termr_23_22_5c_5c_29_22_23(&'input str),
        Termr_23_22_5c_5c_5b_22_23(&'input str),
        Termr_23_22_5c_5c_5d_22_23(&'input str),
        NtCollection(GoCollection),
        NtGameTree(GameTree),
        NtGameTree_2a(::std::vec::Vec<GameTree>),
        NtGameTree_2b(::std::vec::Vec<GameTree>),
        NtNode(Node),
        NtNode_2b(::std::vec::Vec<Node>),
        NtProperty((String, String)),
        NtProperty_2a(::std::vec::Vec<(String, String)>),
        NtProperty_2b(::std::vec::Vec<(String, String)>),
        NtSequence(::std::vec::Vec<Node>),
        Nt____Collection(GoCollection),
    }
    const __ACTION: &'static [i32] = &[
        // State 0
        0, 0, 5, 0, 0, 0,
        // State 1
        0, 0, 0, 0, 0, 0,
        // State 2
        0, 0, -7, -7, 0, 0,
        // State 3
        0, 0, 5, 0, 0, 0,
        // State 4
        10, 0, 0, 0, 0, 0,
        // State 5
        0, 0, -8, -8, 0, 0,
        // State 6
        -11, 0, -11, -11, 0, 0,
        // State 7
        10, 0, -18, -18, 0, 0,
        // State 8
        0, 0, 5, 13, 0, 0,
        // State 9
        -9, 16, -9, -9, 0, 0,
        // State 10
        -12, 0, -12, -12, 0, 0,
        // State 11
        0, 0, 5, 17, 0, 0,
        // State 12
        0, 0, -3, -3, 0, 0,
        // State 13
        -16, -16, -16, -16, 0, 0,
        // State 14
        -10, 16, -10, -10, 0, 0,
        // State 15
        -13, -13, -13, -13, 0, 0,
        // State 16
        0, 0, -4, -4, 0, 0,
        // State 17
        -17, -17, -17, -17, 0, 0,
    ];
    const __EOF_ACTION: &'static [i32] = &[
        // State 0
        -1,
        // State 1
        -19,
        // State 2
        -7,
        // State 3
        -2,
        // State 4
        0,
        // State 5
        -8,
        // State 6
        0,
        // State 7
        0,
        // State 8
        0,
        // State 9
        0,
        // State 10
        0,
        // State 11
        0,
        // State 12
        -3,
        // State 13
        0,
        // State 14
        0,
        // State 15
        0,
        // State 16
        -4,
        // State 17
        0,
    ];
    const __GOTO: &'static [i32] = &[
        // State 0
        2, 3, 0, 4, 0, 0, 0, 0, 0, 0, 0,
        // State 1
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 2
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 3
        0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 4
        0, 0, 0, 0, 7, 8, 0, 0, 0, 9, 0,
        // State 5
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 6
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 7
        0, 0, 0, 0, 11, 0, 0, 0, 0, 0, 0,
        // State 8
        0, 3, 0, 12, 0, 0, 0, 0, 0, 0, 0,
        // State 9
        0, 0, 0, 0, 0, 0, 14, 0, 15, 0, 0,
        // State 10
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 11
        0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 12
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 13
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 14
        0, 0, 0, 0, 0, 0, 18, 0, 0, 0, 0,
        // State 15
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 16
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 17
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    fn __expected_tokens(__state: usize) -> Vec<::std::string::String> {
        const __TERMINAL: &'static [&'static str] = &[
            r###"r#";"#"###,
            r###"r#"[A-Z]+\\[[^\\]]*\\]"#"###,
            r###"r#"\\("#"###,
            r###"r#"\\)"#"###,
            r###"r#"\\["#"###,
            r###"r#"\\]"#"###,
        ];
        __ACTION[(__state * 6)..].iter().zip(__TERMINAL).filter_map(|(&state, terminal)| {
            if state == 0 {
                None
            } else {
                Some(terminal.to_string())
            }
        }).collect()
    }
    #[allow(dead_code)]
    pub fn parse_Collection<
        'input,
    >(
        input: &'input str,
    ) -> Result<GoCollection, __lalrpop_util::ParseError<usize, Token<'input>, &'static str>>
    {
        let mut __tokens = super::__intern_token::__Matcher::new(input);
        let mut __states = vec![0_i32];
        let mut __symbols = vec![];
        let mut __integer;
        let mut __lookahead;
        let __last_location = &mut Default::default();
        '__shift: loop {
            __lookahead = match __tokens.next() {
                Some(Ok(v)) => v,
                None => break '__shift,
                Some(Err(e)) => return Err(e),
            };
            *__last_location = __lookahead.2.clone();
            __integer = match __lookahead.1 {
                Token(1, _) if true => 0,
                Token(0, _) if true => 1,
                Token(2, _) if true => 2,
                Token(3, _) if true => 3,
                Token(4, _) if true => 4,
                Token(5, _) if true => 5,
                _ => {
                    let __state = *__states.last().unwrap() as usize;
                    let __error = __lalrpop_util::ParseError::UnrecognizedToken {
                        token: Some(__lookahead),
                        expected: __expected_tokens(__state),
                    };
                    return Err(__error);
                }
            };
            '__inner: loop {
                let __state = *__states.last().unwrap() as usize;
                let __action = __ACTION[__state * 6 + __integer];
                if __action > 0 {
                    let __symbol = match __integer {
                        0 => match __lookahead.1 {
                            Token(1, __tok0) => __Symbol::Termr_23_22_3b_22_23((__tok0)),
                            _ => unreachable!(),
                        },
                        1 => match __lookahead.1 {
                            Token(0, __tok0) => __Symbol::Termr_23_22_5bA_2dZ_5d_2b_5c_5c_5b_5b_5e_5c_5c_5d_5d_2a_5c_5c_5d_22_23((__tok0)),
                            _ => unreachable!(),
                        },
                        2 => match __lookahead.1 {
                            Token(2, __tok0) => __Symbol::Termr_23_22_5c_5c_28_22_23((__tok0)),
                            _ => unreachable!(),
                        },
                        3 => match __lookahead.1 {
                            Token(3, __tok0) => __Symbol::Termr_23_22_5c_5c_29_22_23((__tok0)),
                            _ => unreachable!(),
                        },
                        4 => match __lookahead.1 {
                            Token(4, __tok0) => __Symbol::Termr_23_22_5c_5c_5b_22_23((__tok0)),
                            _ => unreachable!(),
                        },
                        5 => match __lookahead.1 {
                            Token(5, __tok0) => __Symbol::Termr_23_22_5c_5c_5d_22_23((__tok0)),
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    };
                    __states.push(__action - 1);
                    __symbols.push((__lookahead.0, __symbol, __lookahead.2));
                    continue '__shift;
                } else if __action < 0 {
                    if let Some(r) = __reduce(input, __action, Some(&__lookahead.0), &mut __states, &mut __symbols, ::std::marker::PhantomData::<()>) {
                        if r.is_err() {
                            return r;
                        }
                        return Err(__lalrpop_util::ParseError::ExtraToken { token: __lookahead });
                    }
                } else {
                    let mut __err_lookahead = Some(__lookahead);
                    let mut __err_integer: Option<usize> = Some(__integer);
                    let __state = *__states.last().unwrap() as usize;
                    let __error = __lalrpop_util::ParseError::UnrecognizedToken {
                        token: __err_lookahead,
                        expected: __expected_tokens(__state),
                    };
                    return Err(__error)
                }
            }
        }
        loop {
            let __state = *__states.last().unwrap() as usize;
            let __action = __EOF_ACTION[__state];
            if __action < 0 {
                if let Some(r) = __reduce(input, __action, None, &mut __states, &mut __symbols, ::std::marker::PhantomData::<()>) {
                    return r;
                }
            } else {
                let mut __err_lookahead = None;
                let mut __err_integer: Option<usize> = None;
                let __state = *__states.last().unwrap() as usize;
                let __error = __lalrpop_util::ParseError::UnrecognizedToken {
                    token: __err_lookahead,
                    expected: __expected_tokens(__state),
                };
                return Err(__error)
            }
        }
    }
    pub fn __reduce<
        'input,
    >(
        input: &'input str,
        __action: i32,
        __lookahead_start: Option<&usize>,
        __states: &mut ::std::vec::Vec<i32>,
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: ::std::marker::PhantomData<()>,
    ) -> Option<Result<GoCollection,__lalrpop_util::ParseError<usize, Token<'input>, &'static str>>>
    {
        let __nonterminal = match -__action {
            1 => {
                // Collection =  => ActionFn(16);
                let __start = __symbols.last().map(|s| s.2.clone()).unwrap_or_default();
                let __end = __lookahead_start.cloned().unwrap_or_else(|| __start.clone());
                let __nt = super::__action16::<>(input, &__start, &__end);
                let __states_len = __states.len();
                __states.truncate(__states_len - 0);
                __symbols.push((__start, __Symbol::NtCollection(__nt), __end));
                0
            }
            2 => {
                // Collection = GameTree+ => ActionFn(17);
                let __sym0 = __pop_NtGameTree_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action17::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtCollection(__nt), __end));
                0
            }
            3 => {
                // GameTree = r#"\\("#, Sequence, r#"\\)"# => ActionFn(18);
                let __sym2 = __pop_Termr_23_22_5c_5c_29_22_23(__symbols);
                let __sym1 = __pop_NtSequence(__symbols);
                let __sym0 = __pop_Termr_23_22_5c_5c_28_22_23(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym2.2.clone();
                let __nt = super::__action18::<>(input, __sym0, __sym1, __sym2);
                let __states_len = __states.len();
                __states.truncate(__states_len - 3);
                __symbols.push((__start, __Symbol::NtGameTree(__nt), __end));
                1
            }
            4 => {
                // GameTree = r#"\\("#, Sequence, GameTree+, r#"\\)"# => ActionFn(19);
                let __sym3 = __pop_Termr_23_22_5c_5c_29_22_23(__symbols);
                let __sym2 = __pop_NtGameTree_2b(__symbols);
                let __sym1 = __pop_NtSequence(__symbols);
                let __sym0 = __pop_Termr_23_22_5c_5c_28_22_23(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym3.2.clone();
                let __nt = super::__action19::<>(input, __sym0, __sym1, __sym2, __sym3);
                let __states_len = __states.len();
                __states.truncate(__states_len - 4);
                __symbols.push((__start, __Symbol::NtGameTree(__nt), __end));
                1
            }
            5 => {
                // GameTree* =  => ActionFn(10);
                let __start = __symbols.last().map(|s| s.2.clone()).unwrap_or_default();
                let __end = __lookahead_start.cloned().unwrap_or_else(|| __start.clone());
                let __nt = super::__action10::<>(input, &__start, &__end);
                let __states_len = __states.len();
                __states.truncate(__states_len - 0);
                __symbols.push((__start, __Symbol::NtGameTree_2a(__nt), __end));
                2
            }
            6 => {
                // GameTree* = GameTree+ => ActionFn(11);
                let __sym0 = __pop_NtGameTree_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action11::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtGameTree_2a(__nt), __end));
                2
            }
            7 => {
                // GameTree+ = GameTree => ActionFn(12);
                let __sym0 = __pop_NtGameTree(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action12::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtGameTree_2b(__nt), __end));
                3
            }
            8 => {
                // GameTree+ = GameTree+, GameTree => ActionFn(13);
                let __sym1 = __pop_NtGameTree(__symbols);
                let __sym0 = __pop_NtGameTree_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action13::<>(input, __sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::NtGameTree_2b(__nt), __end));
                3
            }
            9 => {
                // Node = r#";"# => ActionFn(20);
                let __sym0 = __pop_Termr_23_22_3b_22_23(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action20::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtNode(__nt), __end));
                4
            }
            10 => {
                // Node = r#";"#, Property+ => ActionFn(21);
                let __sym1 = __pop_NtProperty_2b(__symbols);
                let __sym0 = __pop_Termr_23_22_3b_22_23(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action21::<>(input, __sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::NtNode(__nt), __end));
                4
            }
            11 => {
                // Node+ = Node => ActionFn(8);
                let __sym0 = __pop_NtNode(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action8::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtNode_2b(__nt), __end));
                5
            }
            12 => {
                // Node+ = Node+, Node => ActionFn(9);
                let __sym1 = __pop_NtNode(__symbols);
                let __sym0 = __pop_NtNode_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action9::<>(input, __sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::NtNode_2b(__nt), __end));
                5
            }
            13 => {
                // Property = r#"[A-Z]+\\[[^\\]]*\\]"# => ActionFn(5);
                let __sym0 = __pop_Termr_23_22_5bA_2dZ_5d_2b_5c_5c_5b_5b_5e_5c_5c_5d_5d_2a_5c_5c_5d_22_23(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action5::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtProperty(__nt), __end));
                6
            }
            14 => {
                // Property* =  => ActionFn(6);
                let __start = __symbols.last().map(|s| s.2.clone()).unwrap_or_default();
                let __end = __lookahead_start.cloned().unwrap_or_else(|| __start.clone());
                let __nt = super::__action6::<>(input, &__start, &__end);
                let __states_len = __states.len();
                __states.truncate(__states_len - 0);
                __symbols.push((__start, __Symbol::NtProperty_2a(__nt), __end));
                7
            }
            15 => {
                // Property* = Property+ => ActionFn(7);
                let __sym0 = __pop_NtProperty_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action7::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtProperty_2a(__nt), __end));
                7
            }
            16 => {
                // Property+ = Property => ActionFn(14);
                let __sym0 = __pop_NtProperty(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action14::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtProperty_2b(__nt), __end));
                8
            }
            17 => {
                // Property+ = Property+, Property => ActionFn(15);
                let __sym1 = __pop_NtProperty(__symbols);
                let __sym0 = __pop_NtProperty_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym1.2.clone();
                let __nt = super::__action15::<>(input, __sym0, __sym1);
                let __states_len = __states.len();
                __states.truncate(__states_len - 2);
                __symbols.push((__start, __Symbol::NtProperty_2b(__nt), __end));
                8
            }
            18 => {
                // Sequence = Node+ => ActionFn(3);
                let __sym0 = __pop_NtNode_2b(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action3::<>(input, __sym0);
                let __states_len = __states.len();
                __states.truncate(__states_len - 1);
                __symbols.push((__start, __Symbol::NtSequence(__nt), __end));
                9
            }
            19 => {
                // __Collection = Collection => ActionFn(0);
                let __sym0 = __pop_NtCollection(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action0::<>(input, __sym0);
                return Some(Ok(__nt));
            }
            _ => panic!("invalid action code {}", __action)
        };
        let __state = *__states.last().unwrap() as usize;
        let __next_state = __GOTO[__state * 11 + __nonterminal] - 1;
        __states.push(__next_state);
        None
    }
    fn __pop_Termr_23_22_3b_22_23<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Termr_23_22_3b_22_23(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Termr_23_22_5bA_2dZ_5d_2b_5c_5c_5b_5b_5e_5c_5c_5d_5d_2a_5c_5c_5d_22_23<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Termr_23_22_5bA_2dZ_5d_2b_5c_5c_5b_5b_5e_5c_5c_5d_5d_2a_5c_5c_5d_22_23(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Termr_23_22_5c_5c_28_22_23<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Termr_23_22_5c_5c_28_22_23(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Termr_23_22_5c_5c_29_22_23<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Termr_23_22_5c_5c_29_22_23(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Termr_23_22_5c_5c_5b_22_23<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Termr_23_22_5c_5c_5b_22_23(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Termr_23_22_5c_5c_5d_22_23<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Termr_23_22_5c_5c_5d_22_23(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtCollection<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, GoCollection, usize)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtCollection(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtGameTree<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, GameTree, usize)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtGameTree(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtGameTree_2a<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, ::std::vec::Vec<GameTree>, usize)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtGameTree_2a(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtGameTree_2b<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, ::std::vec::Vec<GameTree>, usize)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtGameTree_2b(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtNode<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Node, usize)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtNode(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtNode_2b<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, ::std::vec::Vec<Node>, usize)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtNode_2b(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtProperty<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, (String, String), usize)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtProperty(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtProperty_2a<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, ::std::vec::Vec<(String, String)>, usize)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtProperty_2a(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtProperty_2b<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, ::std::vec::Vec<(String, String)>, usize)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtProperty_2b(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_NtSequence<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, ::std::vec::Vec<Node>, usize)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::NtSequence(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
    fn __pop_Nt____Collection<
      'input,
    >(
        __symbols: &mut ::std::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, GoCollection, usize)
     {
        match __symbols.pop().unwrap() {
            (__l, __Symbol::Nt____Collection(__v), __r) => (__l, __v, __r),
            _ => panic!("symbol type mismatch")
        }
    }
}
pub use self::__parse__Collection::parse_Collection;
mod __intern_token {
    #![allow(unused_imports)]
    use std::str::FromStr;
    use gosgf::*;
    use std::collections::HashMap;
    use regex;
    #[allow(unused_extern_crates)]
    extern crate lalrpop_util as __lalrpop_util;
    extern crate regex as __regex;
    use std::fmt as __fmt;

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Token<'input>(pub usize, pub &'input str);
    impl<'a> __fmt::Display for Token<'a> {
        fn fmt(&self, formatter: &mut __fmt::Formatter) -> Result<(), __fmt::Error> {
            __fmt::Display::fmt(self.1, formatter)
        }
    }

    pub struct __Matcher<'input> {
        text: &'input str,
        consumed: usize,
        regex_set: __regex::RegexSet,
        regex_vec: Vec<__regex::Regex>,
    }

    impl<'input> __Matcher<'input> {
        pub fn new(s: &'input str) -> __Matcher<'input> {
            let __strs: &[&str] = &[
                "^(?u:[A-Z])+(?u:\\[)(?u:[\u{0}-\\\\\\^-\u{10ffff}])*(?u:\\])",
                "^(?u:;)",
                "^(?u:\\()",
                "^(?u:\\))",
                "^(?u:\\[)",
                "^(?u:\\])",
            ];
            let __regex_set = __regex::RegexSet::new(__strs).unwrap();
            let __regex_vec = vec![
                __regex::Regex::new("^(?u:[A-Z])+(?u:\\[)(?u:[\u{0}-\\\\\\^-\u{10ffff}])*(?u:\\])").unwrap(),
                __regex::Regex::new("^(?u:;)").unwrap(),
                __regex::Regex::new("^(?u:\\()").unwrap(),
                __regex::Regex::new("^(?u:\\))").unwrap(),
                __regex::Regex::new("^(?u:\\[)").unwrap(),
                __regex::Regex::new("^(?u:\\])").unwrap(),
            ];
            __Matcher {
                text: s,
                consumed: 0,
                regex_set: __regex_set,
                regex_vec: __regex_vec,
            }
        }
    }

    impl<'input> Iterator for __Matcher<'input> {
        type Item = Result<(usize, Token<'input>, usize), __lalrpop_util::ParseError<usize,Token<'input>,&'static str>>;

        fn next(&mut self) -> Option<Self::Item> {
            let __text = self.text.trim_left();
            let __whitespace = self.text.len() - __text.len();
            let __start_offset = self.consumed + __whitespace;
            if __text.is_empty() {
                self.text = __text;
                self.consumed = __start_offset;
                None
            } else {
                let __matches = self.regex_set.matches(__text);
                if !__matches.matched_any() {
                    Some(Err(__lalrpop_util::ParseError::InvalidToken {
                        location: __start_offset,
                    }))
                } else {
                    let mut __longest_match = 0;
                    let mut __index = 0;
                    for __i in 0 .. 6 {
                        if __matches.matched(__i) {
                            let __match = self.regex_vec[__i].find(__text).unwrap();
                            let __len = __match.end();
                            if __len >= __longest_match {
                                __longest_match = __len;
                                __index = __i;
                            }
                        }
                    }
                    let __result = &__text[..__longest_match];
                    let __remaining = &__text[__longest_match..];
                    let __end_offset = __start_offset + __longest_match;
                    self.text = __remaining;
                    self.consumed = __end_offset;
                    Some(Ok((__start_offset, Token(__index, __result), __end_offset)))
                }
            }
        }
    }
}
pub use self::__intern_token::Token;

#[allow(unused_variables)]
fn __action0<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, GoCollection, usize),
) -> GoCollection
{
    (__0)
}

#[allow(unused_variables)]
fn __action1<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, ::std::vec::Vec<GameTree>, usize),
) -> GoCollection
{
    (__0)
}

#[allow(unused_variables)]
fn __action2<
    'input,
>(
    input: &'input str,
    (_, _, _): (usize, &'input str, usize),
    (_, sequence, _): (usize, ::std::vec::Vec<Node>, usize),
    (_, children, _): (usize, ::std::vec::Vec<GameTree>, usize),
    (_, _, _): (usize, &'input str, usize),
) -> GameTree
{
    {

        let komi = f64::from_str(sequence[0].properties.get("KM").unwrap_or(&"0.0".to_owned())).unwrap();
        let size = usize::from_str(sequence[0].properties.get("SZ").unwrap_or(&"19".to_owned())).unwrap();
        
        let handicap;
        {
            let mut handistr = String::from("0");
            for node in &sequence {
                if let Some(ha) = node.properties.get("HA") {
                    handistr = ha.to_string();
                    break;
                }
            }

            handicap =  usize::from_str(&handistr).unwrap();
        }
        
        GameTree {
            komi,
            size,
            handicap,
            sequence,
            children,
        }
    }
}

#[allow(unused_variables)]
fn __action3<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, ::std::vec::Vec<Node>, usize),
) -> ::std::vec::Vec<Node>
{
    (__0)
}

#[allow(unused_variables)]
fn __action4<
    'input,
>(
    input: &'input str,
    (_, _, _): (usize, &'input str, usize),
    (_, pairs, _): (usize, ::std::vec::Vec<(String, String)>, usize),
) -> Node
{
    {
        let mut properties : HashMap<String, String> = HashMap::new();
        for (k, v) in pairs {
            properties.insert(k, v);
        }
        Node { properties }
    }
}

#[allow(unused_variables)]
fn __action5<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> (String, String)
{
    {
        lazy_static! {
            static ref RE : regex::Regex = regex::Regex::new(r"([A-Z]+)\[([^\]]*)\]").unwrap();
        }

        let cap = RE.captures(__0).unwrap();

        let k = &cap[1];
        let v = &cap[2];
        (k.to_string(), v.to_string())
    }
}

#[allow(unused_variables)]
fn __action6<
    'input,
>(
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> ::std::vec::Vec<(String, String)>
{
    vec![]
}

#[allow(unused_variables)]
fn __action7<
    'input,
>(
    input: &'input str,
    (_, v, _): (usize, ::std::vec::Vec<(String, String)>, usize),
) -> ::std::vec::Vec<(String, String)>
{
    v
}

#[allow(unused_variables)]
fn __action8<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, Node, usize),
) -> ::std::vec::Vec<Node>
{
    vec![__0]
}

#[allow(unused_variables)]
fn __action9<
    'input,
>(
    input: &'input str,
    (_, v, _): (usize, ::std::vec::Vec<Node>, usize),
    (_, e, _): (usize, Node, usize),
) -> ::std::vec::Vec<Node>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
fn __action10<
    'input,
>(
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> ::std::vec::Vec<GameTree>
{
    vec![]
}

#[allow(unused_variables)]
fn __action11<
    'input,
>(
    input: &'input str,
    (_, v, _): (usize, ::std::vec::Vec<GameTree>, usize),
) -> ::std::vec::Vec<GameTree>
{
    v
}

#[allow(unused_variables)]
fn __action12<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, GameTree, usize),
) -> ::std::vec::Vec<GameTree>
{
    vec![__0]
}

#[allow(unused_variables)]
fn __action13<
    'input,
>(
    input: &'input str,
    (_, v, _): (usize, ::std::vec::Vec<GameTree>, usize),
    (_, e, _): (usize, GameTree, usize),
) -> ::std::vec::Vec<GameTree>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
fn __action14<
    'input,
>(
    input: &'input str,
    (_, __0, _): (usize, (String, String), usize),
) -> ::std::vec::Vec<(String, String)>
{
    vec![__0]
}

#[allow(unused_variables)]
fn __action15<
    'input,
>(
    input: &'input str,
    (_, v, _): (usize, ::std::vec::Vec<(String, String)>, usize),
    (_, e, _): (usize, (String, String), usize),
) -> ::std::vec::Vec<(String, String)>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
fn __action16<
    'input,
>(
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> GoCollection
{
    let __start0 = __lookbehind.clone();
    let __end0 = __lookahead.clone();
    let __temp0 = __action10(
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action1(
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action17<
    'input,
>(
    input: &'input str,
    __0: (usize, ::std::vec::Vec<GameTree>, usize),
) -> GoCollection
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action11(
        input,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action1(
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action18<
    'input,
>(
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, ::std::vec::Vec<Node>, usize),
    __2: (usize, &'input str, usize),
) -> GameTree
{
    let __start0 = __1.2.clone();
    let __end0 = __2.0.clone();
    let __temp0 = __action10(
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action2(
        input,
        __0,
        __1,
        __temp0,
        __2,
    )
}

#[allow(unused_variables)]
fn __action19<
    'input,
>(
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, ::std::vec::Vec<Node>, usize),
    __2: (usize, ::std::vec::Vec<GameTree>, usize),
    __3: (usize, &'input str, usize),
) -> GameTree
{
    let __start0 = __2.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action11(
        input,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action2(
        input,
        __0,
        __1,
        __temp0,
        __3,
    )
}

#[allow(unused_variables)]
fn __action20<
    'input,
>(
    input: &'input str,
    __0: (usize, &'input str, usize),
) -> Node
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action6(
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action4(
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action21<
    'input,
>(
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, ::std::vec::Vec<(String, String)>, usize),
) -> Node
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action7(
        input,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action4(
        input,
        __0,
        __temp0,
    )
}

pub trait __ToTriple<'input, > {
    type Error;
    fn to_triple(value: Self) -> Result<(usize,Token<'input>,usize),Self::Error>;
}

impl<'input, > __ToTriple<'input, > for (usize, Token<'input>, usize) {
    type Error = &'static str;
    fn to_triple(value: Self) -> Result<(usize,Token<'input>,usize),&'static str> {
        Ok(value)
    }
}
impl<'input, > __ToTriple<'input, > for Result<(usize, Token<'input>, usize),&'static str> {
    type Error = &'static str;
    fn to_triple(value: Self) -> Result<(usize,Token<'input>,usize),&'static str> {
        value
    }
}
