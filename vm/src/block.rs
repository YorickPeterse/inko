//! Executable blocks of code.
//!
//! A Block is an executable block of code (based on a CompiledCode) combined
//! with binding of the scope the block was created in.
use binding::RcBinding;
use compiled_code::RcCompiledCode;
use global_scope::GlobalScopeReference;

#[derive(Clone)]
pub struct Block {
    /// The CompiledCode containing the instructions to run.
    pub code: RcCompiledCode,

    /// The binding of the scope in which this block was created.
    pub binding: RcBinding,

    /// The global scope this block belongs to.
    pub global_scope: GlobalScopeReference,
}

impl Block {
    pub fn new(code: RcCompiledCode,
               binding: RcBinding,
               global_scope: GlobalScopeReference)
               -> Self {
        Block {
            code: code,
            binding: binding,
            global_scope: global_scope,
        }
    }

    #[inline(always)]
    pub fn arguments(&self) -> usize {
        self.code.arguments as usize
    }

    #[inline(always)]
    pub fn required_arguments(&self) -> usize {
        self.code.required_arguments as usize
    }

    pub fn locals(&self) -> usize {
        self.code.locals as usize
    }

    pub fn has_rest_argument(&self) -> bool {
        self.code.rest_argument
    }

    pub fn name(&self) -> &String {
        &self.code.name
    }
}