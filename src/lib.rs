//! An outline of how Rust's ownership, borrowing and lifetimes could be combined with JS-managed data
//!
//! The goals are:
//! 
//! 1. Ensure that JS objects are only accessed in the right JS compartment.
//! 2. Support stack-allocated roots.
//! 3. Remove the need for the rooting lint.
//! 4. Don't require rooting in code that can't perform GC.
//! 5. Allow `&mut T` access to JS-managed data, so we don't need as much interior mutability.
//!
//! # JS-managed data
//!
//! The idea is that Rust data can be given to JS to manage, and then accessed,
//! using the JS context. This is passed as a variable of type `JSContext<S>`,
//! where the type parameter `S` is used to track the state of the context.
//!
//! For example, we can give JS some Rust data to manage in compartment
//! `C` when the context state implements the `CanAlloc` trait:
//!
//! ```rust
//! # use linjs::*;
//! fn example<C, S>(cx: &mut JSContext<S>) where
//!     S: CanAlloc + InCompartment<C>,
//! {
//!     let x: JSManaged<C, String> = cx.manage(String::from("hello"));
//! }
//! ```
//!
//! JS-managed data in compartment `C` can be accessed if the context state
//! implements the `CanAccess<C>` trait:
//!
//! ```rust
//! # use linjs::*;
//! fn example<C, S>(cx: &mut JSContext<S>, x: JSManaged<C, String>) where
//!     S: CanAccess + InCompartment<C>,
//! {
//!     println!("{} world", x.get(cx));
//! }
//! ```
//!
//! # Lifetimes of JS-managed data
//!
//! Unfortunately, combining these two examples is not memory-safe, due to
//! garbage collection:
//!
//! ```rust,ignore
//! # use linjs::*;
//! fn unsafe_example<C, S>(cx: &mut JSContext<S>) where
//!     S: CanAlloc + CanAccess + InCompartment<C>,
//! {
//!     let x: JSManaged<C, String> = cx.manage(String::from("hello"));
//!     // Imagine something triggers GC here
//!     println!("{} world", x.get(cx));
//! }
//! ```
//!
//! This example is not safe, as there is nothing keeping `x` alive in JS,
//! so if garbage collection is triggered, then `x` will be reclaimed
//! which will drop the Rust data, and so the call to `x.get(cx)` will be a use-after-free.
//!
//! This example is not memory-safe, and fortunately fails to typecheck:
//!
//! ```text
//! 	error[E0502]: cannot borrow `*cx` as immutable because it is also borrowed as mutable
//!  --> <anon>:7:32
//!   |
//! 5 |     let x: JSManaged<C, String> = cx.manage(String::from("hello"));
//!   |                                   -- mutable borrow occurs here
//! 6 |     // Imagine something triggers GC here
//! 7 |     println!("{} world", x.get(cx));
//!   |                                ^^ immutable borrow occurs here
//! 8 | }
//!   | - mutable borrow ends here
//! ```
//!
//! To see why this example fails to typecheck, we can introduce explicit lifetimes:
//!
//! ```rust,ignore
//! # use linjs::*;
//! fn unsafe_example<'a, C, S>(cx: &'a mut JSContext<S>) where
//!     S: CanAlloc + CanAccess + InCompartment<C>,
//! {
//!     // x has type JSManaged<'b, C, String>
//!     let x = cx.manage(String::from("hello"));
//!     // Imagine something triggers GC here
//!     // x_ref has type &'c String
//!     let x_ref = x.get(cx);
//!     println!("{} world", x_ref);
//! }
//! ```
//!
//! We can now see why this fails to typecheck: since `cx` is borrowed mutably at type
//! `&'b mut JSContext<S>`, then immutably at type `&'c mut JSContext<S>` these lifetimes
//! cannot overlap, but the call to `x.get(cx)` requires them to overlap. These contradicting
//! constraints cause the example to fail to compile.
//!
//! # Rooting
//!
//! To fix this example, we need to make sure that `x` lives long enough. One way to do this is
//! to root `x`, so that it will not be garbage collected. 
//!
//! ```rust
//! # use linjs::*;
//! fn example<'a, C: 'a, S>(cx: &'a mut JSContext<S>) where
//!     S: CanAlloc + CanAccess + InCompartment<C>,
//! {
//!     // Function body has lifetime 'b
//!     // x has type JSManaged<'b, C, String>
//!     rooted!(in(cx) let x = cx.manage(String::from("hello")));
//!     // Imagine something triggers GC here
//!     // x_ref has type &'c String
//!     let x_ref = x.get(cx);
//!     println!("{} world", x_ref);
//! }
//! ```
//!
//! This example is now safe, since `x` is rooted during its access.
//! The example typechecks because the root has lifetime `'b`, and there is
//! no constraint that `'b` and `'c` don't overlap.
//! This use of lifetimes allows safe access to JS-managed data without a special
//! rooting lint.
//!
//! We can root any data which implements the `JSRootable` and `JSTraceable` traits,
//! which includes JS-managed data. These traits can be derived,
//! using the `#[derive(JSRootable, JSTraceable)]` type annotation.
//!
//! # Mutating JS-managed data
//!
//! JS managed data can be accessed mutably as well as immutably.
//! This is safe because mutably accessing JS manage data requires
//! mutably borrowing the JS context, so there cannot be two simultaneous
//! mutable accesses.
//!
//! ```rust
//! # use linjs::*;
//! fn example<C, S>(cx: &mut JSContext<S>, x: JSManaged<C, String>) where
//!     S: CanAccess + InCompartment<C>,
//! {
//!     println!("{} world", x.get(cx));
//!     *x.get_mut(cx) = String::from("hi");
//!     println!("{} world", x.get(cx));
//! }
//! ```
//!
//! An attempt to mutably access JS managed data more than once simultaneously
//! results in an error from the borrow-checker, for example:
//!
//! ```rust,ignore
//! # use linjs::*; use std::mem;
//! fn unsafe_example<C, S>(cx: &mut JSContext<S>, x: JSManaged<C, String>, y: JSManaged<C, String>) where
//!     S: CanAccess + InCompartment<C>,
//! {
//!     mem::swap(x.get_mut(cx), y.get_mut(cx));
//! }
//! ```
//!
//! ```text
//!         error[E0499]: cannot borrow `*cx` as mutable more than once at a time
//!  --> <anon>:7:40
//!   |
//! 7 |     mem::swap(x.get_mut(cx), y.get_mut(cx));
//!   |                         --             ^^ - first borrow ends here
//!   |                         |              |
//!   |                         |              second mutable borrow occurs here
//!   |                         first mutable borrow occurs here
//! ```
//!
//! Mutable update allows the construction of cyclic structures, for example:
//!
//! ```rust
//! # #[macro_use] extern crate linjs;
//! # #[macro_use] extern crate linjs_derive;
//! # use linjs::*;
//! #[derive(JSTraceable, JSRootable)]
//! struct NativeLoop<'a, C> {
//!    next: Option<Loop<'a, C>>,
//! }
//! impl<'a, C> HasClass for NativeLoop<'a, C> { type Class = LoopClass; }
//! type Loop<'a, C> = JSManaged<'a, C, LoopClass>;
//! struct LoopClass;
//! impl<'a, C> HasInstance<'a, C> for LoopClass { type Instance = NativeLoop<'a, C>; }
//! fn example<C, S>(cx: &mut JSContext<S>) where
//!     S: CanAccess + CanAlloc + InCompartment<C>,
//! {
//!    rooted!(in(cx) let l = cx.manage(NativeLoop { next: None }));
//!    l.get_mut(cx).next = Some(l);
//! }
//! # fn main() {}
//! ```
//!
//! # Snapshots
//!
//! Some cases of building JS managed data require rooting, but in some cases
//! the rooting can be avoided, since the program does nothing to trigger
//! garbage collection. In this case, we can snapshot the JS context after
//! performing allocation. The snapshot supports accessing JS managed data,
//! but does not support any calls that might trigger garbage collection.
//! As a result, we know that any data which is live at the beginning of
//! the snapshot is also live at the end.
//!
//! ```rust
//! # extern crate linjs;
//! # #[macro_use] extern crate linjs_derive;
//! # use linjs::*;
//! # #[derive(JSTraceable, JSRootable)]
//! # struct NativeLoop<'a, C> {
//! #    next: Option<Loop<'a, C>>,
//! # }
//! # impl<'a, C> HasClass for NativeLoop<'a, C> { type Class = LoopClass; }
//! # type Loop<'a, C> = JSManaged<'a, C, LoopClass>;
//! # struct LoopClass;
//! # impl<'a, C> HasInstance<'a, C> for LoopClass { type Instance = NativeLoop<'a, C>; }
//! fn example<C, S>(cx: &mut JSContext<S>) where
//!     S: CanAccess + CanAlloc + InCompartment<C>,
//! {
//!    let (ref mut cx, l) = cx.snapshot_manage(NativeLoop { next: None });
//!    l.get_mut(cx).next = Some(l);
//! }
//! # fn main() {}
//! ```
//!
//! A program which tries to use a function which might trigger GC will
//! not typecheck, as the snapshotted JS context state does not support
//! the appropriate traits. For example:
//!
//! ```rust,ignore
//! # #[macro_use] extern crate linjs;
//! # #[macro_use] extern crate linjs_derive;
//! # use linjs::*;
//! # #[derive(JSTraceable, JSRootable)]
//! # struct NativeLoop<'a, C> {
//! #    next: Option<Loop<'a, C>>,
//! # }
//! # type Loop<'a, C> = JSManaged<'a, C, NativeLoop<'a, C>>;
//! fn might_trigger_gc<C, S>(cx: &mut JSContext<S>) where
//!     S: CanAccess + CanAlloc + InCompartment<C>
//! { }
//!
//! fn unsafe_example<C, S>(cx: &mut JSContext<S>) where
//!     S: CanAccess + CanAlloc + InCompartment<C>
//! {
//!    let (ref mut cx, l) = cx.snapshot_manage(NativeLoop { next: None });
//!    might_trigger_gc(cx);
//!    l.get_mut(cx).next = Some(l);
//! }
//! # fn main() {}
//! ```
//!
//! In this program, the function `might_trigger_gc` requires the state
//! to support `CanAlloc<C>`, which is not allowed by the snapshotted state.
//! 
//! ```text
//! 	error[E0277]: the trait bound `linjs::Snapshotted<'_, S>: linjs::CanAlloc<C>` is not satisfied
//!   --> <anon>:16:4
//!    |
//! 16 |    might_trigger_gc(cx);
//!    |    ^^^^^^^^^^^^^^^^ the trait `linjs::CanAlloc<C>` is not implemented for `linjs::Snapshotted<'_, S>`
//!    |
//!    = note: required by `might_trigger_gc`
//! ```
//!
//! # Globals
//!
//! JS contexts require initialization. In particular, each compartment has a global,
//! which should be JS managed data. The global can be initialized using `cx.init(value)`,
//! which updates the state of the context from uninitialized to initialized.
//!
//! ```rust
//! # extern crate linjs;
//! # #[macro_use] extern crate linjs_derive;
//! # use linjs::*;
//! #[derive(JSTraceable, JSRootable)]
//! struct NativeMyGlobal { name: String }
//! impl HasClass for NativeMyGlobal { type Class = MyGlobalClass; }
//!
//! struct MyGlobalClass;
//! impl<'a, C> HasInstance<'a, C> for MyGlobalClass { type Instance = NativeMyGlobal; }
//!
//! type MyGlobal<'a, C> = JSManaged<'a, C, NativeMyGlobal>;
//!
//! fn example<'a, C, S>(cx: JSContext<S>) -> JSContext<Initialized<C>> where
//!    S: CanCreate<C>,
//!    C: HasGlobal<MyGlobalClass>,
//! {
//!    let cx = cx.create_compartment();
//!    let name = String::from("Alice");
//!    cx.global_manage(NativeMyGlobal { name: name })
//! }
//! # fn main() {}
//! ```
//!
//! The current global can be accessed from the JS context, for example:
//!
//! ```rust
//! # #[macro_use] extern crate linjs;
//! # #[macro_use] extern crate linjs_derive;
//! # use linjs::*;
//! #[derive(JSTraceable, JSRootable)]
//! # struct NativeMyGlobal { name: String }
//! # impl HasClass for NativeMyGlobal { type Class = MyGlobalClass; }
//! # type MyGlobal<'a, C> = JSManaged<'a, C, NativeMyGlobal>;
//! # type MyContext<'a, C> = JSContext<Initialized<MyGlobal<'a, C>>>;
//! # struct MyGlobalClass;
//! # impl<'a, C> HasInstance<'a, C> for MyGlobalClass { type Instance = NativeMyGlobal; }
//! #
//! fn example<'a, C, S>(cx: &JSContext<S>) where
//!    S: CanAccess + InCompartment<C>,
//!    C: HasGlobal<MyGlobalClass>,
//! {
//!    println!("My global is named {}.", cx.global().get(cx).name);
//! }
//! # fn main() {}
//! ```
//!
//! In some cases, the global contains some JS-managed data, which is why the initialization
//! is split into two steps: creating the compartment, and
//! providing the JS-managed data for the global, for example:
//!
//! ```rust
//! # #[macro_use] extern crate linjs;
//! # #[macro_use] extern crate linjs_derive;
//! # use linjs::*;
//! #[derive(JSTraceable, JSRootable)]
//! struct NativeMyGlobal<'a, C> { name: JSManaged<'a, C, String> }
//! impl<'a, C> HasClass for NativeMyGlobal<'a, C> { type Class = MyGlobalClass; }
//!
//! struct MyGlobalClass;
//! impl<'a, C> HasInstance<'a, C> for MyGlobalClass { type Instance = NativeMyGlobal<'a, C>; }
//!
//! fn example<'a, C, S>(cx: JSContext<S>) -> JSContext<Initialized<C>> where
//!    S: CanCreate<C>,
//!    C: HasGlobal<MyGlobalClass>,
//! {
//!    let mut cx = cx.create_compartment();
//!    rooted!(in(cx) let name = cx.manage(String::from("Alice")));
//!    cx.global_manage(NativeMyGlobal { name: name })
//! }
//! # fn main() {}
//! ```
//!
//! During initialization, it is safe to perform allocation, but
//! not much else, as the global is still uninitialized.
//! For example:
//!
//! ```rust,ignore
//! # #[macro_use] extern crate linjs;
//! # #[macro_use] extern crate linjs_derive;
//! # use linjs::*;
//! # #[derive(JSTraceable, JSRootable)]
//! # struct NativeMyGlobal<'a, C> { name: JSManaged<'a, C, String> }
//! # impl<'a,C> HasClass for NativeMyGlobal<'a,C> {}
//! # type MyGlobal<'a, C> = JSManaged<'a, C, NativeMyGlobal<'a, C>>;
//! # type MyContext<'a, C> = JSContext<Initialized<MyGlobal<'a, C>>>;
//! #
//! fn unsafe_example<'a, C, S>(cx: JSContext<S>) -> MyContext<'a, C> where
//!    C: 'a,
//!    S: CanInitialize + InCompartment<C>,
//! {
//!    let mut cx = cx.pre_init();
//!    let oops = cx.global().get(&cx).name.get(&cx);
//!    rooted!(in(cx) let name = cx.manage(String::from("Alice")));
//!    cx.post_init(NativeMyGlobal { name: name })
//! }
//! # fn main() {}
//! ```
//!
//! This code is unsafe, since the global is accessed before it is initialized,
//! but does not typecheck because the context state does not allow accessing
//! JS-managed data during initialization.
//!
//! ```text
//! 	error[E0277]: the trait bound `linjs::Initializing<linjs::JSManaged<'_, C, _>>: linjs::CanAccess<C>` is not satisfied
//!   --> <anon>:14:27
//!    |
//! 14 |    let oops = cx.global().get(&cx).name.get(&cx);
//!    |                           ^^^ the trait `linjs::CanAccess<C>` is not implemented for `linjs::Initializing<linjs::JSManaged<'_, C, _>>`
//! ```
//!
//! # Bootstrapping
//!
//! To bootstrap initialization, a user defines a type which implements the `JSRunnable` trait.
//! This requires a `run` method, which takes the JS context as an argument.  The `JSRunnable`
//! trait provides a `start()` method which calls the `run(cx)` method back with an appropriate
//! context.
//!
//! ```rust
//! # extern crate linjs;
//! # #[macro_use] extern crate linjs_derive;
//! # use linjs::*;
//! #[derive(JSTraceable, JSRootable)]
//! struct NativeMyGlobal { name: String }
//! impl HasClass for NativeMyGlobal { type Class = MyGlobalClass; }
//!
//! struct MyGlobalClass;
//! impl<'a, C> HasInstance<'a, C> for MyGlobalClass { type Instance = NativeMyGlobal; }
//!
//! struct Example;
//!
//! impl JSRunnable<MyGlobalClass> for Example {
//!     fn run<C, S>(self, cx: JSContext<S>) where
//!         S: CanCreate<C>,
//!         C: HasGlobal<MyGlobalClass>,
//!     {
//!         let cx = cx.create_compartment();
//!         let name = String::from("Alice");
//!         let ref cx = cx.global_manage(NativeMyGlobal { name: name });
//!         assert_eq!(cx.global().get(cx).name, "Alice");
//!     }
//! }
//!
//! fn main() { Example.start(); }
//! ```
//!
//! #Examples
//!
//! This is an example of building a two-node cyclic graph, which is the smallest
//! example that Rust would need `Rc` and `RefCell` for. Note that this builds
//! the graph with no need for rooting.
//!
//! ```
//! #[macro_use] extern crate linjs;
//! #[macro_use] extern crate linjs_derive;
//! use linjs::{CanAlloc, CanAccess, CanCreate, CanExtend};
//! use linjs::{HasClass, HasGlobal, HasInstance, InCompartment};
//! use linjs::{JSContext, JSManaged, JSRunnable};
//!
//! // A graph type
//! type Graph<'a, C> = JSManaged<'a, C, GraphClass>;
//! #[derive(JSTraceable, JSRootable)]
//! struct NativeGraph<'a, C> {
//!     nodes: Vec<Node<'a, C>>,
//! }
//! impl<'a, C> HasClass for NativeGraph<'a, C> { type Class = GraphClass; }
//!
//! struct GraphClass;
//! impl<'a, C> HasInstance<'a, C> for GraphClass { type Instance = NativeGraph<'a, C>; }
//!
//! // A node type
//! type Node<'a, C> = JSManaged<'a, C, NodeClass>;
//! #[derive(JSTraceable, JSRootable)]
//! struct NativeNode<'a, C> {
//!     data: usize,
//!     edges: Vec<Node<'a, C>>,
//! }
//! impl<'a, C> HasClass for NativeNode<'a, C> { type Class = NodeClass; }
//!
//! struct NodeClass;
//! impl<'a, C> HasInstance<'a, C> for NodeClass { type Instance = NativeNode<'a, C>; }
//!
//! // Build a cyclic graph
//! struct Example;
//! impl JSRunnable<GraphClass> for Example {
//!     fn run<C, S>(self, cx: JSContext<S>) where
//!         S: CanCreate<C>,
//!         C: HasGlobal<GraphClass>,
//!     {
//!         let cx = cx.create_compartment();
//!         let ref mut cx = cx.global_manage(NativeGraph { nodes: vec![] });
//!         let graph = cx.global();
//!         self.add_node1(cx, graph);
//!         self.add_node2(cx, graph);
//!         assert_eq!(graph.get(cx).nodes[0].get(cx).data, 1);
//!         assert_eq!(graph.get(cx).nodes[1].get(cx).data, 2);
//!         let ref mut cx = cx.snapshot();
//!         self.add_edges(cx, graph);
//!         assert_eq!(graph.get(cx).nodes[0].get(cx).edges[0].get(cx).data, 2);
//!         assert_eq!(graph.get(cx).nodes[1].get(cx).edges[0].get(cx).data, 1);
//!     }
//! }
//!
//! impl Example {
//!     fn add_node1<S, C>(&self, cx: &mut JSContext<S>, graph: Graph<C>) where
//!         S: CanAccess + CanAlloc + InCompartment<C>
//!     {
//!         // Creating nodes does memory allocation, which may trigger GC,
//!         // so we need to be careful about lifetimes while they are being added.
//!         // Approach 1 is to root the node.
//!         rooted!(in(cx) let node1 = cx.manage(NativeNode { data: 1, edges: vec![] }));
//!         graph.get_mut(cx).nodes.push(node1);
//!     }
//!     fn add_node2<S, C>(&self, cx: &mut JSContext<S>, graph: Graph<C>) where
//!         S: CanAccess + CanAlloc + InCompartment<C>
//!      {
//!         // Approach 2 is to take a snapshot of the context right after allocation.
//!         let (ref mut cx, node2) = cx.snapshot_manage(NativeNode { data: 2, edges: vec![] });
//!         graph.get_mut(cx).nodes.push(node2);
//!     }
//!     fn add_edges<'a, S, C>(&self, cx: &mut JSContext<S>, graph: Graph<C>) where
//!         C: 'a,
//!         S: CanAccess + CanExtend<'a> + InCompartment<C>
//!      {
//!         // Note that there's no rooting here.
//!         let node1 = graph.get(cx).nodes[0].extend_lifetime(cx);
//!         let node2 = graph.get(cx).nodes[1].extend_lifetime(cx);
//!         node1.get_mut(cx).edges.push(node2);
//!         node2.get_mut(cx).edges.push(node1);
//!     }
//! }
//!
//! fn main() { Example.start(); }
//! ```

#![feature(associated_type_defaults)]
#![feature(const_fn)]
    
extern crate js;
extern crate libc;

use js::jsapi;
use js::jsapi::HandleObject;
use js::jsapi::JSClass;
use js::jsapi::JSClassOps;
use js::jsapi::JSNative;
use js::jsapi::JSNativeWrapper;
use js::jsapi::JSFunctionSpec;
use js::jsapi::JSPropertySpec;
use js::jsapi::JS_InitClass;
use js::jsapi::JS_InitStandardClasses;

pub use js::jsapi::JSTracer;

use libc::c_char;
use libc::c_uint;

use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ptr;

/// The type for JS contexts whose current state is `S`.
pub struct JSContext<S> {
    global_raw: *mut (),
    marker: PhantomData<S>,
}

/// A context state in an initialized compartment `C`.
pub struct Initialized<C> (PhantomData<C>);

/// A context state which can initialize compartment `C`.
pub struct Uninitialized<C> (PhantomData<C>);

/// A context state in the middle of initializing a compartment `C`.
pub struct Initializing<C> (PhantomData<C>);

/// A context state in snapshotted compartment in underlying state `S`,
/// which guarantees that no GC will happen during the lifetime `'a`.
pub struct Snapshotted<'a, S> (PhantomData<(&'a(), S)>);

/// A marker trait for JS contexts in compartment `C`
pub trait InCompartment<C> {}
impl<C> InCompartment<C> for Initializing<C> {}
impl<C> InCompartment<C> for Initialized<C> {}
impl<'a, C, S> InCompartment<C> for Snapshotted<'a, S> where S: InCompartment<C> {}

/// A marker trait for JS contexts that can access native state
pub trait CanAccess {}
impl<C> CanAccess for Initialized<C> {}
impl<'a, S> CanAccess for Snapshotted<'a, S> where S: CanAccess {}

/// A marker trait for JS contexts that can extend the lifetime of objects
pub trait CanExtend<'a> {}
impl<'a, S> CanExtend<'a> for Snapshotted<'a, S> {}

/// A marker trait for JS contexts that can allocate objects
pub trait CanAlloc {}
impl<G> CanAlloc for Initialized<G> {}
impl<G> CanAlloc for Initializing<G> {}

/// A marker trait for JS contexts that can create compartment `C`.
pub trait CanCreate<C> {}
impl<C> CanCreate<C> for Uninitialized<C> {}

/// A marker trait for JS contexts that are in the middle of initializing
pub trait IsInitializing {}
impl<C> IsInitializing for Initializing<C> {}

/// A marker trait for JS compartments that have a global of class `K`.
pub trait HasGlobal<K> {}

impl<S> JSContext<S> {
    /// Get a snapshot of the JS state.
    /// The snapshot only allows access to the methods that are guaranteed not to call GC,
    /// so we don't need to root JS-managed pointers during the lifetime of a snapshot.
    pub fn snapshot<'a>(&'a mut self) -> JSContext<Snapshotted<'a, S>> {
        JSContext {
            global_raw: self.global_raw,
            marker: PhantomData,
        }
    }

    /// Give ownership of data to JS.
    /// This allocates JS heap, which may trigger GC.
    pub fn manage<'a, 'b, C, K, T>(&'a mut self, value: T) -> JSManaged<'a, C, K> where
        S: CanAlloc + InCompartment<C>,
        T: HasClass<Class = K>,
        K: HasInstance<'b, C, Instance = T>,
    {
        // The real thing would use a JS reflector to manage the space,
        // this just space-leaks
        JSManaged {
            raw: Box::into_raw(Box::new(value)) as *mut (),
            marker: PhantomData,
        }
    }

    /// Give ownership of data to JS.
    /// This allocates JS heap, which may trigger GC.
    pub fn snapshot_manage<'a, 'b, C, K, T>(&'a mut self, value: T) -> (JSContext<Snapshotted<'a, S>>, JSManaged<'a, C, K>) where
        S: CanAlloc + InCompartment<C>,
        T: HasClass<Class = K>,
        K: HasInstance<'b, C, Instance = T>,
    {
        // The real thing would use a JS reflector to manage the space,
        // this just space-leaks
        let managed = JSManaged {
            raw: Box::into_raw(Box::new(value)) as *mut (),
            marker: PhantomData,
        };
        let snapshot = JSContext {
            global_raw: self.global_raw,
            marker: PhantomData,
        };
        (snapshot, managed)
    }

    /// Create a compartment
    pub fn create_compartment<'a, C, K>(self) -> JSContext<Initializing<C>> where
        S: CanCreate<C>,
        C: HasGlobal<K>,
        K: HasInstance<'a, C>,
    {
        // This is dangerous!
        // This is only safe because dereferencing this pointer is only done by user code
        // in posession of a context whose state is `CanAccess`. The only way a user can
        // access such a context is by calling `post_init`, which initializes the raw pointer.
        // TODO: check that `Drop` and GC tracing are safe.
        // TODO: check the performance of the safer version of this code, which stores an `Option<T>` rather than a `T`.
        // TODO: hook into jsapi compartment creation
        let boxed: Box<K::Instance> = unsafe { Box::new(mem::uninitialized()) };
        let raw = Box::into_raw(boxed) as *mut ();
        JSContext {
            global_raw: raw,
            marker: PhantomData,
        }
    }

    /// Finish initializing a JS Context
    pub fn global_manage<'a, 'b, C, K, T>(self, value: T) -> JSContext<Initialized<C>> where
        S: IsInitializing + InCompartment<C>,
        C: HasGlobal<K>,
        K: HasInstance<'b, C, Instance = T>,
        T: HasClass<Class = K>,
    {
        let raw = self.global_raw as *mut T;
        let uninitialized = unsafe { mem::replace(&mut *raw, value) };
        mem::forget(uninitialized);
        JSContext {
            global_raw: self.global_raw,
            marker: PhantomData,
        }
    }

    /// Get the global of an initialized context.
    pub fn global<'a, 'b, C, G>(&'a self) -> JSManaged<'b, C, G> where
        S: InCompartment<C>,
        C: HasGlobal<G>,
    {
        JSManaged {
            raw: self.global_raw,
            marker: PhantomData,
        }
    }

    /// Create a new root.
    pub fn new_root<T>(&mut self) -> JSRoot<T> {
        JSRoot {
            value: None,
            pin: JSUntypedPinnedRoot {
                value: unsafe { mem::zeroed() },
                next: ptr::null_mut(),
                prev: ptr::null_mut(),
            },
        }
    }

    // A real implementation would also have JS methods such as those in jsapi.
}

/// A trait for Rust data that can be traced.
pub unsafe trait JSTraceable {
    unsafe fn trace(&self, trc: *mut JSTracer);

    fn as_ptr(&self) -> *const JSTraceable where Self: Sized {
        unsafe { mem::transmute(self as &JSTraceable) }
    }

    fn as_mut_ptr(&mut self) -> *mut JSTraceable where Self: Sized {
        unsafe { mem::transmute(self as &mut JSTraceable) }
    }
}

unsafe impl JSTraceable for String {
    unsafe fn trace(&self, _trc: *mut JSTracer) {}
}

unsafe impl JSTraceable for usize {
    unsafe fn trace(&self, _trc: *mut JSTracer) {}
}

unsafe impl<T> JSTraceable for Option<T> where T: JSTraceable {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        if let Some(ref val) = *self {
            val.trace(trc);
        }
    }
}

unsafe impl<T> JSTraceable for Vec<T> where T: JSTraceable {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        for val in self {
            val.trace(trc);
        }
    }
}

// etc.

/// A trait for Rust data which can be reflected

pub trait HasClass {
    type Class;
    type Init: JSInitializer = DefaultInitializer;
}

pub trait HasInstance<'a, C>: Sized {
    type Instance: HasClass<Class = Self>;
}

/// Basic types
impl HasClass for String { type Class = String; }
impl<'a, C> HasInstance<'a, C> for String { type Instance = String; }

/// Initialize JS data

pub trait JSInitializer {
    unsafe fn parent_prototype() -> HandleObject {
        HandleObject::null()
    }
    
    unsafe fn classp() -> *const JSClass {
        &DEFAULT_CLASS
    }

    unsafe fn constructor() -> (JSNative, c_uint) {
        (None, 0)
    }

    unsafe fn properties() -> *const JSPropertySpec {
        ptr::null()
    }

    unsafe fn functions() -> *const JSFunctionSpec {
        ptr::null()
    }

    unsafe fn static_properties() -> *const JSPropertySpec {
        ptr::null()
    }

    unsafe fn static_functions() -> *const JSFunctionSpec {
        ptr::null()
    }

    unsafe fn js_init_class(cx: *mut jsapi::JSContext, global: jsapi::HandleObject) {
        let parent_proto = Self::parent_prototype();
        let classp = Self::classp();
        let (constructor, nargs) = Self::constructor();
        let ps = Self::properties();
        let fs = Self::functions();
        let static_ps = Self::static_properties();
        let static_fs = Self::static_functions();
        JS_InitClass(cx, global, parent_proto, classp, constructor, nargs, ps, fs, static_ps, static_fs);
    }

    unsafe fn js_init_object(_cx: *mut jsapi::JSContext, _obj: jsapi::HandleObject) {
    }

    unsafe fn js_init_global(cx: *mut jsapi::JSContext, global: jsapi::HandleObject) {
        JS_InitStandardClasses(cx, global);
    }
}

/// A default class.

pub struct DefaultInitializer;

impl JSInitializer for DefaultInitializer {}

static DEFAULT_CLASS: JSClass = JSClass {
    name: b"[Object]\0" as *const u8 as *const c_char,
    flags: 0,
    cOps: &JSClassOps {
        addProperty: None,
        call: None,
        construct: None,
        delProperty: None,
        enumerate: None,
        finalize: None,
        getProperty: None,
        hasInstance: None,
        mayResolve: None,
        resolve: None,
        setProperty: None,
        trace: None,
    },
    reserved: [0 as *mut _; 3],
};

pub const fn null_wrapper() -> JSNativeWrapper {
    JSNativeWrapper {
        op: None,
        info: ptr::null(),
    }
}

pub const fn null_property() -> JSPropertySpec {
    JSPropertySpec {
        name: ptr::null(),
        flags: 0,
        getter: null_wrapper(),
        setter: null_wrapper(),
    }
}

pub const fn null_function() -> JSFunctionSpec {
    JSFunctionSpec {
        name: ptr::null(),
        flags: 0,
        call: null_wrapper(),
        nargs: 0,
        selfHostedName: ptr::null(),
    }
}

/// A trait for a Rust class.

pub trait HasJSClass {
    fn js_class() -> &'static JSClass;
}

/// A user of a JS runtime implements `JSRunnable`.
pub trait JSRunnable<K>: Sized {
    /// This callback is called with a fresh JS compartment type `C`.
    fn run<C, S>(self, cx: JSContext<S>) where
        S: CanCreate<C>,
        C: HasGlobal<K>;

    /// To trigger the callback, call `rt.start()`.
    fn start(self) {
        struct JSCompartmentImpl;
        impl<K> HasGlobal<K> for JSCompartmentImpl {}
        let cx = JSContext {
            global_raw: ptr::null_mut(),
            marker: PhantomData,
        };
        self.run::<JSCompartmentImpl, Uninitialized<JSCompartmentImpl>>(cx);
    }
}



/// The type of JS-managed data in a JS compartment `C`, with lifetime `'a` and class `K`.
///
/// If the user has access to a `JSManaged`, then the JS-managed
/// data is live for the given lifetime.
pub struct JSManaged<'a, C, K> {
    // JS reflector goes here
    raw: *mut (),
    marker: PhantomData<(&'a(), C, K)>
}

impl<'a, C, K> Clone for JSManaged<'a, C, K> where
    K: HasInstance<'a, C>,    
{
    fn clone(&self) -> Self {
        JSManaged {
            raw: self.raw,
            marker: PhantomData,
        }
    }
}

impl<'a, C, K> Copy for JSManaged<'a, C, K> where
    K: HasInstance<'a, C>,
{
}

unsafe impl<'a, C, K> JSTraceable for JSManaged<'a, C, K> where
    K: HasInstance<'a, C>,
    K::Instance: JSTraceable,
{
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Trace the reflector.
    }
}

impl<'a, C, K> JSManaged<'a, C, K> {
    /// Read-only access to JS-managed data.
    pub fn get<'b, S>(self, _: &'b JSContext<S>) -> &'b K::Instance where
        S: CanAccess,
        K: HasInstance<'b, C>,
        'a: 'b,
    {
        unsafe { &*(self.raw as *mut K::Instance) }
    }

    /// Read-write access to JS-managed data.
    pub fn get_mut<'b, S>(self, _: &'b mut JSContext<S>) -> &'b mut K::Instance where
        S: CanAccess,
        K: HasInstance<'b, C>,
        'a: 'b,
    {
        unsafe { &mut *(self.raw as *mut K::Instance) }
    }

    /// Change the lifetime of JS-managed data.
    pub unsafe fn change_lifetime<'b>(self) -> JSManaged<'b, C, K> {
        JSManaged {
            raw: self.raw,
            marker: PhantomData,
        }
    }

    /// It's safe to extend the lifetime of JS-managed data if it has been snapshotted.
    pub fn extend_lifetime<'b, 'c, S>(self, _: &'c JSContext<S>) -> JSManaged<'b, C, K> where
        S: CanExtend<'b>,
    {
        unsafe { self.change_lifetime() }
    }
}

/// A stack allocated root containing data of type `T`.`
pub struct JSRoot<T> {
    value: Option<T>,
    pin: JSUntypedPinnedRoot,
}

/// A stack allocated root that has been pinned, so the backing store can't move.
pub struct JSPinnedRoot<'a, T: 'a> (&'a mut JSRoot<T>);

/// A doubly linked list with all the pinned roots.
#[derive(Eq, PartialEq)]
pub struct JSPinnedRoots(*mut JSUntypedPinnedRoot);

/// The thread-local list of all roots
thread_local! { static ROOTS: UnsafeCell<JSPinnedRoots> = UnsafeCell::new(JSPinnedRoots(ptr::null_mut())); }

unsafe fn thread_local_roots() -> *mut JSPinnedRoots {
    ROOTS.with(UnsafeCell::get)
}

/// A stack allocated root that has been pinned, but we don't have a type for the contents
struct JSUntypedPinnedRoot {
    value: *mut JSTraceable,
    next: *mut JSUntypedPinnedRoot,
    prev: *mut JSUntypedPinnedRoot,
}

/// Data which can be rooted.

pub unsafe trait JSRootable<'a> {
    type Aged;

    unsafe fn change_lifetime(self) -> Self::Aged where Self: Sized {
        let result = mem::transmute_copy(&self);
        mem::forget(self);
        result
    }

    fn contract_lifetime(self) -> Self::Aged where Self: 'a + Sized {
        unsafe { self.change_lifetime() }
    }
}

unsafe impl<'a, 'b, C, K> JSRootable<'a> for JSManaged<'b, C, K>
{
    type Aged = JSManaged<'a, C, K>;
}

impl<T> JSRoot<T> {
    // Very annoyingly, this function has to be marked as unsafe,
    // because we can't rely on the destructor for the pinned root running.
    // See the discussion about `mem::forget` being safe at
    // https://github.com/rust-lang/rfcs/pull/1066.
    // This is safe as long as it is unpinned before the memory
    // is reclaimed, but Rust does not enforce that.
    pub unsafe fn pin<'a, U>(&'a mut self, value: U) -> JSPinnedRoot<'a, T> where
        T: JSTraceable,
        U: JSRootable<'a, Aged=T>,
    {
        let roots = thread_local_roots();
        self.value = Some(value.change_lifetime());
        self.pin.value = self.value.as_mut_ptr();
        self.pin.next = (*roots).0;
        self.pin.prev = ptr::null_mut();
        if let Some(next) = self.pin.next.as_mut() {
            next.prev = &mut self.pin;
        }
        *roots = JSPinnedRoots(&mut self.pin);
        JSPinnedRoot(self)
    }

    pub unsafe fn unpin(&mut self) {
        if let Some(next) = self.pin.next.as_mut() {
            next.prev = self.pin.prev;
        }
        if let Some(prev) = self.pin.prev.as_mut() {
            prev.next = self.pin.next;
        } else {
            let roots = thread_local_roots();
            *roots = JSPinnedRoots(self.pin.next);
        }
        self.value = None;
        self.pin.value = mem::zeroed();
        self.pin.next = ptr::null_mut();
        self.pin.prev = ptr::null_mut();
    }
}

impl<'a, T> Deref for JSPinnedRoot<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0.value.as_ref().unwrap()
    }
}

impl<'a, T> DerefMut for JSPinnedRoot<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0.value.as_mut().unwrap()
    }
}

impl<'a, T> Drop for JSPinnedRoot<'a, T> {
    fn drop(&mut self) {
        unsafe { self.0.unpin() }
    }
}

#[macro_export]
macro_rules! rooted {
    (in($cx:expr) let $name:ident = $init:expr) => (
        let mut __root = $cx.new_root();
        #[allow(unsafe_code)]
        let __pinned = unsafe { __root.pin($init) };
        let $name = *__pinned;
    );
    (in($cx:expr) let mut $name:ident = $init:expr) => (
        let mut __root = $cx.new_root();
        #[allow(unsafe_code)]
        let __pinned = unsafe { __root.pin($init) };
        let mut $name = *__pinned;
    );
    (in($cx:expr) let ref $name:ident = $init:expr) => (
        let mut __root = $cx.new_root();
        #[allow(unsafe_code)]
        let __pinned = unsafe { __root.pin($init) };
        let $name = &*__pinned;
    );
    (in($cx:expr) let ref mut $name:ident = $init:expr) => (
        let mut __root = $cx.new_root();
        #[allow(unsafe_code)]
        let __pinned = unsafe { __root.pin($init) };
        let mut $name = &mut*__pinned;
    )
}
