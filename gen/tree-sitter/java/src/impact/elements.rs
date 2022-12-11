// use std::{
//     borrow::Borrow,
//     collections::{HashMap, HashSet},
//     fmt::{Debug, Display},
//     hash::Hash,
//     ops::{Deref, Index},
// };

// use bitvec::order::Lsb0;
// use enumset::{enum_set, EnumSet, EnumSetType};
// use string_interner::{DefaultSymbol, StringInterner, Symbol};

// use crate::impact::element::Arguments;

// use super::{label_value::LabelValue, element::{RefsEnum, RefPtr, LabelPtr, Nodes, ExplorableRef, self}, java_element::Primitive, reference::{DisplayRef, self}, declaration::{Declarator, DeclType, self, DisplayDecl}};


// fn default_imports<F: FnMut(&str) -> LabelPtr>(solver: &mut Solver, mut intern_label: F) {
//     macro_rules! scoped {
//         ( $o:expr, $i:expr ) => {{
//             let o = $o;
//             let i = $i;
//             solver.intern(RefsEnum::ScopedIdentifier(o, i))
//         }};
//     }
//     macro_rules! import {
//         ( $($p:expr),* ) => {
//             {
//                 let t = solver.intern(RefsEnum::Root);
//                 $(
//                     let i = intern_label($p);
//                     let t = scoped!(t, i);
//                 )*
//                 let i = scoped!(solver.intern(RefsEnum::MaybeMissing), i);
//                 let d = Declarator::Type(i);
//                 solver.add_decl_simple(d, t);
//             }
//         }
//     }
//     // import!("java","lang","Appendable");
//     // import!("java","lang","AutoCloseable");
//     // import!("java","lang","CharSequence");
//     // import!("java","lang","Cloneable");
//     // import!("java","lang","Comparable");//<T>
//     // import!("java","lang","Iterable");//<T>
//     // import!("java","lang","Readable");
//     // import!("java","lang","Runnable");
//     // import!("java","lang","Thread","UncaughtExceptionHandler");
//     // import!("java","lang","Byte");
//     // import!("java","lang","Character");
//     // import!("java","lang","Character","Subset");
//     // import!("java","lang","Character","UnicodeBlock");
//     // import!("java","lang","Class");//<T>
//     // import!("java","lang","ClassLoader");
//     // import!("java","lang","ClassValue");//<T>
//     // import!("java","lang","Compiler");
//     // import!("java","lang","Double");
//     // import!("java","lang","Enum"); //<E extends Enum<E>>
//     // import!("java","lang","Float");
//     // import!("java","lang","InheritableThreadLocal");//<T>
//     // import!("java", "lang", "Integer");
//     // import!("java","lang","Long");
//     // import!("java","lang","Math");
//     // import!("java","lang","Number");
//     // import!("java","lang","Object");
//     // import!("java","lang","Package");
//     // import!("java","lang","Process");
//     // import!("java","lang","ProcessBuilder");
//     // import!("java","lang","ProcessBuilder","Redirect");
//     // import!("java","lang","Runtime");
//     // import!("java","lang","RuntimePermission");
//     // import!("java","lang","SecurityManager");
//     // import!("java","lang","Short");
//     // import!("java","lang","StackTraceElement");
//     // import!("java","lang","StrictMath");
//     // import!("java", "lang", "String");
//     // import!("java","lang","StringBuffer");
//     // import!("java","lang","StringBuilder");
//     // import!("java","lang","System");
//     // import!("java","lang","Thread");
//     // import!("java","lang","ThreadGroup");
//     // import!("java","lang","ThreadLocal");//<T>
//     // import!("java","lang","Throwable");
//     // import!("java","lang","Void");
//     // import!("java","lang","ProcessBuilder","Redirect","Type");
//     // import!("java","lang","Thread","State");
//     // import!("java","lang","ArrayIndexOutOfBoundsException");
//     // import!("java","lang","ArrayStoreException");
//     // import!("java","lang","ClassCastException");
//     // import!("java","lang","ClassNotFoundException");
//     // import!("java","lang","CloneNotSupportedException");
//     // import!("java","lang","EnumConstantNotPresentException");
//     // import!("java","lang","Exception");
//     // import!("java","lang","IllegalAccessException");
//     // import!("java","lang","IllegalArgumentException");
//     // import!("java","lang","IllegalMonitorStateException");
//     // import!("java","lang","IllegalStateException");
//     // import!("java","lang","IllegalThreadStateException");
//     // import!("java","lang","IndexOutOfBoundsException");
//     // import!("java","lang","InstantiationException");
//     // import!("java","lang","InterruptedException");
//     // import!("java","lang","NegativeArraySizeException");
//     // import!("java","lang","NoSuchFieldException");
//     // import!("java","lang","NoSuchMethodException");
//     // import!("java","lang","NullPointerException");
//     // import!("java","lang","NumberFormatException");
//     // import!("java","lang","ReflectiveOperationException");
//     // import!("java","lang","RuntimeException");
//     // import!("java","lang","SecurityException");
//     // import!("java","lang","StringIndexOutOfBoundsException");
//     // import!("java","lang","TypeNotPresentException");
//     // import!("java","lang","UnsupportedOperationException");
//     // import!("java","lang","AssertionError");
//     // import!("java","lang","BootstrapMethodError");
//     // import!("java","lang","ClassCircularityError");
//     // import!("java","lang","ClassFormatError");
//     // import!("java","lang","Error");
//     // import!("java","lang","ExceptionInInitializerError");
//     // import!("java","lang","IllegalAccessError");
//     // import!("java","lang","IncompatibleClassChangeError");
//     // import!("java","lang","InstantiationError");
//     // import!("java","lang","InternalError");
//     // import!("java","lang","LinkageError");
//     // import!("java","lang","NoClassDefFoundError");
//     // import!("java","lang","NoSuchFieldError");
//     // import!("java","lang","NoSuchMethodError");
//     // import!("java","lang","OutOfMemoryError");
//     // import!("java","lang","StackOverflowError");
//     // import!("java","lang","ThreadDeath");
//     // import!("java","lang","UnknownError");
//     // import!("java","lang","UnsatisfiedLinkError");
//     // import!("java","lang","UnsupportedClassVersionError");
//     // import!("java","lang","VerifyError");
//     // import!("java","lang","VirtualMachineError");
//     // import!("java","lang","Override");
//     // import!("java","lang","SafeVarargs");
//     // import!("java","lang","SuppressWarnings");
// }

// trait RefMap {
//     fn get<Q: Borrow<RefPtr>>(&self, k: Q) -> Option<&RefPtr>;
//     fn insert(&mut self, k: RefPtr, v: RefPtr);
// }

// struct RefHashMap {
//     map: HashMap<RefPtr, RefPtr>,
// }

// impl RefMap for RefHashMap {
//     fn get<Q: Borrow<RefPtr>>(&self, k: Q) -> Option<&RefPtr> {
//         self.map.get(k.borrow())
//     }

//     fn insert(&mut self, k: RefPtr, v: RefPtr) {
//         self.map.insert(k, v);
//     }
// }

// enum Simplify<T> {
//     Continue(T),
//     Terminate(T),
// }

// trait RefMapSimp {
//     fn get<Q: Borrow<RefPtr>>(&self, k: Q) -> Option<Simplify<&RefPtr>>;
//     fn insert(&mut self, k: RefPtr, v: Simplify<RefPtr>);
// }

// struct RefHashMapSimp {
//     map: HashMap<RefPtr, RefPtr>,
//     terminate: bitvec::vec::BitVec,
// }

// impl RefMapSimp for RefHashMapSimp {
//     fn get<Q: Borrow<RefPtr>>(&self, k: Q) -> Option<Simplify<&RefPtr>> {
//         let r = self.map.get(k.borrow());
//         if let Some(r) = r {
//             let r = if self.terminate.len() > *k.borrow() && self.terminate[*k.borrow()] {
//                 Simplify::Terminate(r)
//             } else {
//                 Simplify::Continue(r)
//             };
//             Some(r)
//         } else {
//             None
//         }
//     }

//     fn insert(&mut self, k: RefPtr, v: Simplify<RefPtr>) {
//         let v = match v {
//             Simplify::Continue(v) => v,
//             Simplify::Terminate(v) => {
//                 if !(self.terminate.len() > k) {
//                     self.terminate.resize(k + 1, false)
//                 }
//                 self.terminate.set(k, true);
//                 v
//             }
//         };
//         self.map.insert(k, v);
//     }
// }
