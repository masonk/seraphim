Rust Best Practices

#1:
    Follow the API guidelines at https://rust-lang-nursery.github.io/api-guidelines/about.html

- Style
    - Don't panic!:
        There is almost never a reason to panic from a library in Rust. When you panic, you take control away from your caller.
        Return a Result instead. If you are using error_chain, returning a Result is just as easy as panicing, and is far more useful.
        - In particular, do not panic if allocation fails. Instead return an Error.
    - Most APIs should return Result:
        Almost all public operations in an API should return a Result, even if the operation cannot presently fail.
        It's easy for a caller to unwrap a Result, but once an API exposes a raw T, it's hard to change it to a Result<T>.
        Backwards compatibility with a public API that exposes a raw types is the greatest tempation for violating guideline #1.
        Be defensive and always return Results from public APIs.  This is extremely easy to do when using the failure crate; see below.
    - Don't import *:
        Optimize for readability rather than writeability. When reading code, it's useful to see exactly which types are imported from another crate. 
        It's especially useful to have this information readily available when trying to remove a crate dependency.
    - For public types, derive what is derivable: 
        These common traits should be derived whenever reasonable:
        - Default, Debug, Display, Clone, PartialEq, Eq, PartialCmp, Cmp, Hash
        Deriving standard traits it easier for consumers to work with your type. These types provide an enormous amount of functionality for almost no cost. 
    - For public types, implement what is implementable:
        Providing implementations for standard traits like:
        Index, IndexMut, Send, Sync, AsRef, AsMut, Borrow, BorrowMut, Into, From, FromStr, TryFrom, Serialize, Deserialize, IntoIterator, Deref, DerefMut
        Again makes it easier to 
    - For public types, be judicious when implementing certain standard types:
        The traits that a public type derives are part of its API, and removing a trait implementation from a type is a breaking change.
        - Copy:
        A type that starts life as Copy may evolve into a type that cannot implement Copy. Thus, public types normally should not implement Copy unless you can predict that the type will always be able to implement Copy.
    - For public functions, accept conversion traits:
        conversion traits are incredibly useful abstractions over concrete types, that allow functions to accept a wider variety of inputs from their callers. 
        - Rather than accept Option<T>, accept std::convert::Into<Option<T>>
        - Rather than accept &T, accept AsRef<T>
        - Rather than accept Iterator<Item=T>, accept IntoIterator<T>
    - For public types, return impl Trait:




- Always use these standard crates:
    - log
        Using log makes your library's logging behavior congruent with its consumer's logging behavior. You should always use it.
    - structured logging:
        https://github.com/rust-lang-nursery/log/blob/master/rfcs/0296-structured-logging.md
    - serde:
        Use serde for all data serialization needs. Doing so makes your types compatible with 
    - failure:
        All libraries should expose Errors via failure from the start. It's just as easy to write error_chain Errors as ad-hoc errors,
        and doing so makes it easy to deal with the errors as structured data.
        https://github.com/rust-lang-nursery/failure
        https://boats.gitlab.io/blog/post/2017-11-16-announcing-failure/
        
    - struct_opt:
        If your library needs configuration, the best practice is to expose an options struct that derives structopt. This allows consuming binaries
        to compose your options configuration into their argument parsing.
    - use clap for argument processing