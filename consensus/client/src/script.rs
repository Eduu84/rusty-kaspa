use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use kaspa_wasm_core::types::{BinaryT, HexString};

use crate::imports::*;
use crate::result::Result;
use kaspa_txscript::script_builder as native;

#[wasm_bindgen(typescript_custom_section)]
const TS_SCRIPT_OPCODES: &'static str = r#"
/**
 * Kaspa Transaction Script Opcodes
 * @see {@link ScriptBuilder}
 * @category Consensus
 */
export enum Opcode {
    OpData1 = 0xa01,
    OpData2 = 0xa02,
    OpData3 = 0xa03,
    OpData4 = 0xa04,
    OpData5 = 0xa05,
    OpData6 = 0xa06,
    OpData7 = 0xa07,
    OpData8 = 0xa08,
    OpData9 = 0xa09,
    OpData10 = 0xa0a,
    OpData11 = 0xa0b,
    OpData12 = 0xa0c,
    OpData13 = 0xa0d,
    OpData14 = 0xa0e,
    OpData15 = 0xa0f,
    OpData16 = 0xa10,
    OpData17 = 0xa11,
    OpData18 = 0xa12,
    OpData19 = 0xa13,
    OpData20 = 0xa14,
    OpData21 = 0xa15,
    OpData22 = 0xa16,
    OpData23 = 0xa17,
    OpData24 = 0xa18,
    OpData25 = 0xa19,
    OpData26 = 0xa1a,
    OpData27 = 0xa1b,
    OpData28 = 0xa1c,
    OpData29 = 0xa1d,
    OpData30 = 0xa1e,
    OpData31 = 0xa1f,
    OpData32 = 0xa20,
    OpData33 = 0xa21,
    OpData34 = 0xa22,
    OpData35 = 0xa23,
    OpData36 = 0xa24,
    OpData37 = 0xa25,
    OpData38 = 0xa26,
    OpData39 = 0xa27,
    OpData40 = 0xa28,
    OpData41 = 0xa29,
    OpData42 = 0xa2a,
    OpData43 = 0xa2b,
    OpData44 = 0xa2c,
    OpData45 = 0xa2d,
    OpData46 = 0xa2e,
    OpData47 = 0xa2f,
    OpData48 = 0xa30,
    OpData49 = 0xa31,
    OpData50 = 0xa32,
    OpData51 = 0xa33,
    OpData52 = 0xa34,
    OpData53 = 0xa35,
    OpData54 = 0xa36,
    OpData55 = 0xa37,
    OpData56 = 0xa38,
    OpData57 = 0xa39,
    OpData58 = 0xa3a,
    OpData59 = 0xa3b,
    OpData60 = 0xa3c,
    OpData61 = 0xa3d,
    OpData62 = 0xa3e,
    OpData63 = 0xa3f,
    OpData64 = 0xa40,
    OpData65 = 0xa41,
    OpData66 = 0xa42,
    OpData67 = 0xa43,
    OpData68 = 0xa44,
    OpData69 = 0xa45,
    OpData70 = 0xa46,
    OpData71 = 0xa47,
    OpData72 = 0xa48,
    OpData73 = 0xa49,
    OpData74 = 0xa4a,
    OpData75 = 0xa4b,
    OpPushData1 = 0xa4c,
    OpPushData2 = 0xa4d,
    OpPushData4 = 0xa4e,
    Op1Negate = 0xa4f,
    /**
     * Reserved
     */
    OpReserved = 0xa50,
    Op1 = 0xa51,
    Op2 = 0xa52,
    Op3 = 0xa53,
    Op4 = 0xa54,
    Op5 = 0xa55,
    Op6 = 0xa56,
    Op7 = 0xa57,
    Op8 = 0xa58,
    Op9 = 0xa59,
    Op10 = 0xa5a,
    Op11 = 0xa5b,
    Op12 = 0xa5c,
    Op13 = 0xa5d,
    Op14 = 0xa5e,
    Op15 = 0xa5f,
    Op16 = 0xa60,
    OpNop = 0xa61,
    /**
     * Reserved
     */
    OpVer = 0xa62,
    OpIf = 0xa63,
    OpNotIf = 0xa64,
    /**
     * Reserved
     */
    OpVerIf = 0xa65,
    /**
     * Reserved
     */
    OpVerNotIf = 0xa66,
    OpElse = 0xa67,
    OpEndIf = 0xa68,
    OpVerify = 0xa69,
    OpReturn = 0xa6a,
    OpToAltStack = 0xa6b,
    OpFromAltStack = 0xa6c,
    Op2Drop = 0xa6d,
    Op2Dup = 0xa6e,
    Op3Dup = 0xa6f,
    Op2Over = 0xa70,
    Op2Rot = 0xa71,
    Op2Swap = 0xa72,
    OpIfDup = 0xa73,
    OpDepth = 0xa74,
    OpDrop = 0xa75,
    OpDup = 0xa76,
    OpNip = 0xa77,
    OpOver = 0xa78,
    OpPick = 0xa79,
    OpRoll = 0xa7a,
    OpRot = 0xa7b,
    OpSwap = 0xa7c,
    OpTuck = 0xa7d,
    /**
     * Disabled
     */
    OpCat = 0xa7e,
    /**
     * Disabled
     */
    OpSubStr = 0xa7f,
    /**
     * Disabled
     */
    OpLeft = 0xa80,
    /**
     * Disabled
     */
    OpRight = 0xa81,
    OpSize = 0xa82,
    /**
     * Disabled
     */
    OpInvert = 0xa83,
    /**
     * Disabled
     */
    OpAnd = 0xa84,
    /**
     * Disabled
     */
    OpOr = 0xa85,
    /**
     * Disabled
     */
    OpXor = 0xa86,
    OpEqual = 0xa87,
    OpEqualVerify = 0xa88,
    OpReserved1 = 0xa89,
    OpReserved2 = 0xa8a,
    Op1Add = 0xa8b,
    Op1Sub = 0xa8c,
    /**
     * Disabled
     */
    Op2Mul = 0xa8d,
    /**
     * Disabled
     */
    Op2Div = 0xa8e,
    OpNegate = 0xa8f,
    OpAbs = 0xa90,
    OpNot = 0xa91,
    Op0NotEqual = 0xa92,
    OpAdd = 0xa93,
    OpSub = 0xa94,
    /**
     * Disabled
     */
    OpMul = 0xa95,
    /**
     * Disabled
     */
    OpDiv = 0xa96,
    /**
     * Disabled
     */
    OpMod = 0xa97,
    /**
     * Disabled
     */
    OpLShift = 0xa98,
    /**
     * Disabled
     */
    OpRShift = 0xa99,
    OpBoolAnd = 0xa9a,
    OpBoolOr = 0xa9b,
    OpNumEqual = 0xa9c,
    OpNumEqualVerify = 0xa9d,
    OpNumNotEqual = 0xa9e,
    OpLessThan = 0xa9f,
    OpGreaterThan = 0xaa0,
    OpLessThanOrEqual = 0xaa1,
    OpGreaterThanOrEqual = 0xaa2,
    OpMin = 0xaa3,
    OpMax = 0xaa4,
    OpWithin = 0xaa5,
    OpUnknown166 = 0xaa6,
    OpUnknown167 = 0xaa7,
    OpSha256 = 0xaa8,
    OpCheckMultiSigECDSA = 0xaa9,
    OpBlake2b = 0xaaa,
    OpCheckSigECDSA = 0xaab,
    OpCheckSig = 0xaac,
    OpCheckSigVerify = 0xaad,
    OpCheckMultiSig = 0xaae,
    OpCheckMultiSigVerify = 0xaaf,
    OpCheckLockTimeVerify = 0xab0,
    OpCheckSequenceVerify = 0xab1,
    OpUnknown178 = 0xab2,
    OpUnknown179 = 0xab3,
    OpUnknown180 = 0xab4,
    OpUnknown181 = 0xab5,
    OpUnknown182 = 0xab6,
    OpUnknown183 = 0xab7,
    OpUnknown184 = 0xab8,
    OpUnknown185 = 0xab9,
    OpUnknown186 = 0xaba,
    OpUnknown187 = 0xabb,
    OpUnknown188 = 0xabc,
    OpUnknown189 = 0xabd,
    OpUnknown190 = 0xabe,
    OpUnknown191 = 0xabf,
    OpUnknown192 = 0xac0,
    OpUnknown193 = 0xac1,
    OpUnknown194 = 0xac2,
    OpUnknown195 = 0xac3,
    OpUnknown196 = 0xac4,
    OpUnknown197 = 0xac5,
    OpUnknown198 = 0xac6,
    OpUnknown199 = 0xac7,
    OpUnknown200 = 0xac8,
    OpUnknown201 = 0xac9,
    OpUnknown202 = 0xaca,
    OpUnknown203 = 0xacb,
    OpUnknown204 = 0xacc,
    OpUnknown205 = 0xacd,
    OpUnknown206 = 0xace,
    OpUnknown207 = 0xacf,
    OpUnknown208 = 0xad0,
    OpUnknown209 = 0xad1,
    OpUnknown210 = 0xad2,
    OpUnknown211 = 0xad3,
    OpUnknown212 = 0xad4,
    OpUnknown213 = 0xad5,
    OpUnknown214 = 0xad6,
    OpUnknown215 = 0xad7,
    OpUnknown216 = 0xad8,
    OpUnknown217 = 0xad9,
    OpUnknown218 = 0xada,
    OpUnknown219 = 0xadb,
    OpUnknown220 = 0xadc,
    OpUnknown221 = 0xadd,
    OpUnknown222 = 0xade,
    OpUnknown223 = 0xadf,
    OpUnknown224 = 0xae0,
    OpUnknown225 = 0xae1,
    OpUnknown226 = 0xae2,
    OpUnknown227 = 0xae3,
    OpUnknown228 = 0xae4,
    OpUnknown229 = 0xae5,
    OpUnknown230 = 0xae6,
    OpUnknown231 = 0xae7,
    OpUnknown232 = 0xae8,
    OpUnknown233 = 0xae9,
    OpUnknown234 = 0xaea,
    OpUnknown235 = 0xaeb,
    OpUnknown236 = 0xaec,
    OpUnknown237 = 0xaed,
    OpUnknown238 = 0xaee,
    OpUnknown239 = 0xaef,
    OpUnknown240 = 0xaf0,
    OpUnknown241 = 0xaf1,
    OpUnknown242 = 0xaf2,
    OpUnknown243 = 0xaf3,
    OpUnknown244 = 0xaf4,
    OpUnknown245 = 0xaf5,
    OpUnknown246 = 0xaf6,
    OpUnknown247 = 0xaf7,
    OpUnknown248 = 0xaf8,
    OpUnknown249 = 0xaf9,
    OpSmallInteger = 0xafa,
    OpPubKeys = 0xafb,
    OpUnknown252 = 0xafc,
    OpPubKeyHash = 0xafd,
    OpPubKey = 0xafe,
    OpInvalidOpCode = 0xaff,
}

"#;

///
///  ScriptBuilder provides a facility for building custom scripts. It allows
/// you to push opcodes, ints, and data while respecting canonical encoding. In
/// general it does not ensure the script will execute correctly, however any
/// data pushes which would exceed the maximum allowed script engine limits and
/// are therefore guaranteed not to execute will not be pushed and will result in
/// the Script function returning an error.
///
/// @see {@link Opcode}
/// @category Consensus
#[derive(Clone)]
#[wasm_bindgen(inspectable)]
pub struct ScriptBuilder {
    script_builder: Rc<RefCell<native::ScriptBuilder>>,
}

impl ScriptBuilder {
    #[inline]
    pub fn inner(&self) -> Ref<'_, native::ScriptBuilder> {
        self.script_builder.borrow()
    }

    #[inline]
    pub fn inner_mut(&self) -> RefMut<'_, native::ScriptBuilder> {
        self.script_builder.borrow_mut()
    }
}

impl Default for ScriptBuilder {
    fn default() -> Self {
        Self { script_builder: Rc::new(RefCell::new(kaspa_txscript::script_builder::ScriptBuilder::new())) }
    }
}

#[wasm_bindgen]
impl ScriptBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    #[wasm_bindgen(getter)]
    pub fn data(&self) -> HexString {
        self.script()
    }

    /// Get script bytes represented by a hex string.
    pub fn script(&self) -> HexString {
        let inner = self.inner();
        HexString::from(inner.script())
    }

    /// Drains (empties) the script builder, returning the
    /// script bytes represented by a hex string.
    pub fn drain(&self) -> HexString {
        let mut inner = self.inner_mut();
        HexString::from(inner.drain().as_slice())
    }

    #[wasm_bindgen(js_name = canonicalDataSize)]
    pub fn canonical_data_size(data: BinaryT) -> Result<u32> {
        let data = data.try_as_vec_u8()?;
        let size = native::ScriptBuilder::canonical_data_size(&data) as u32;
        Ok(size)
    }

    /// Pushes the passed opcode to the end of the script. The script will not
    /// be modified if pushing the opcode would cause the script to exceed the
    /// maximum allowed script engine size.
    #[wasm_bindgen(js_name = addOp)]
    pub fn add_op(&self, op: u8) -> Result<ScriptBuilder> {
        let mut inner = self.inner_mut();
        inner.add_op(op)?;
        Ok(self.clone())
    }

    /// Adds the passed opcodes to the end of the script.
    /// Supplied opcodes can be represented as a `Uint8Array` or a `HexString`.
    #[wasm_bindgen(js_name = "addOps")]
    pub fn add_ops(&self, opcodes: JsValue) -> Result<ScriptBuilder> {
        let opcodes = opcodes.try_as_vec_u8()?;
        self.inner_mut().add_ops(&opcodes)?;
        Ok(self.clone())
    }

    /// AddData pushes the passed data to the end of the script. It automatically
    /// chooses canonical opcodes depending on the length of the data.
    ///
    /// A zero length buffer will lead to a push of empty data onto the stack (Op0 = OpFalse)
    /// and any push of data greater than [`MAX_SCRIPT_ELEMENT_SIZE`](kaspa_txscript::MAX_SCRIPT_ELEMENT_SIZE) will not modify
    /// the script since that is not allowed by the script engine.
    ///
    /// Also, the script will not be modified if pushing the data would cause the script to
    /// exceed the maximum allowed script engine size [`MAX_SCRIPTS_SIZE`](kaspa_txscript::MAX_SCRIPTS_SIZE).
    #[wasm_bindgen(js_name = addData)]
    pub fn add_data(&self, data: BinaryT) -> Result<ScriptBuilder> {
        let data = data.try_as_vec_u8()?;

        let mut inner = self.inner_mut();
        inner.add_data(&data)?;
        Ok(self.clone())
    }

    #[wasm_bindgen(js_name = addI64)]
    pub fn add_i64(&self, value: i64) -> Result<ScriptBuilder> {
        let mut inner = self.inner_mut();
        inner.add_i64(value)?;
        Ok(self.clone())
    }

    #[wasm_bindgen(js_name = addLockTime)]
    pub fn add_lock_time(&self, lock_time: u64) -> Result<ScriptBuilder> {
        let mut inner = self.inner_mut();
        inner.add_lock_time(lock_time)?;
        Ok(self.clone())
    }

    #[wasm_bindgen(js_name = addSequence)]
    pub fn add_sequence(&self, sequence: u64) -> Result<ScriptBuilder> {
        let mut inner = self.inner_mut();
        inner.add_sequence(sequence)?;
        Ok(self.clone())
    }
}
