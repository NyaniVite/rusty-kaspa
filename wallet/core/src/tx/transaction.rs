use super::input::TransactionInput;
use super::output::TransactionOutput;
use super::payment::PaymentOutputs;
use crate::imports::*;
use crate::Result;
use workflow_wasm::abi::ref_from_abi;
use workflow_wasm::jsvalue::JsValueTrait;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionInner {
    pub version: u16,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub lock_time: u64,
    pub subnetwork_id: SubnetworkId,
    pub gas: u64,
    pub payload: Vec<u8>,

    // A field that is used to cache the transaction ID.
    // Always use the corresponding self.id() instead of accessing this field directly
    pub id: TransactionId,
}

/// Represents a Kaspa transaction
#[derive(Clone, Debug, Serialize, Deserialize)]
#[wasm_bindgen(inspectable)]
pub struct Transaction {
    inner: Arc<Mutex<TransactionInner>>,
}

impl Transaction {
    pub fn new(
        version: u16,
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
        lock_time: u64,
        subnetwork_id: SubnetworkId,
        gas: u64,
        payload: Vec<u8>,
    ) -> Result<Self> {
        let tx = Self {
            inner: Arc::new(Mutex::new(TransactionInner {
                version,
                inputs,
                outputs,
                lock_time,
                subnetwork_id,
                gas,
                payload,
                id: Default::default(), // Temp init before the finalize below
            })),
        };
        tx.finalize()?;
        Ok(tx)
    }

    pub fn new_with_inner(inner: TransactionInner) -> Self {
        Self { inner: Arc::new(Mutex::new(inner)) }
    }

    pub fn inner(&self) -> MutexGuard<'_, TransactionInner> {
        self.inner.lock().unwrap()
    }

    pub fn id(&self) -> TransactionId {
        self.inner().id
    }
}

#[wasm_bindgen]
impl Transaction {
    /// Determines whether or not a transaction is a coinbase transaction. A coinbase
    /// transaction is a special transaction created by miners that distributes fees and block subsidy
    /// to the previous blocks' miners, and specifies the script_pub_key that will be used to pay the current
    /// miner in future blocks.
    pub fn is_coinbase(&self) -> bool {
        self.inner().subnetwork_id == subnets::SUBNETWORK_ID_COINBASE
    }

    /// Recompute and finalize the tx id based on updated tx fields
    pub fn finalize(&self) -> Result<TransactionId> {
        let tx: cctx::Transaction = self.try_into()?;
        self.inner().id = tx.id();
        Ok(self.inner().id)
    }

    /// Returns the transaction ID
    #[wasm_bindgen(getter, js_name = id)]
    pub fn id_string(&self) -> String {
        self.inner().id.to_string()
    }

    #[wasm_bindgen(constructor)]
    pub fn constructor(js_value: JsValue) -> std::result::Result<Transaction, JsError> {
        Ok(js_value.try_into()?)
    }

    #[wasm_bindgen(getter = inputs)]
    pub fn get_inputs_as_js_array(&self) -> Array {
        let inputs = self.inner.lock().unwrap().inputs.clone().into_iter().map(JsValue::from);
        Array::from_iter(inputs)
    }

    #[wasm_bindgen(setter = inputs)]
    pub fn set_inputs_from_js_array(&mut self, js_value: &JsValue) {
        let inputs = Array::from(js_value)
            .iter()
            .map(|js_value| {
                ref_from_abi!(TransactionInput, &js_value).unwrap_or_else(|err| panic!("invalid transaction input: {err}"))
            })
            .collect::<Vec<_>>();
        self.inner().inputs = inputs;
    }

    #[wasm_bindgen(getter = outputs)]
    pub fn get_outputs_as_js_array(&self) -> Array {
        let outputs = self.inner.lock().unwrap().outputs.clone().into_iter().map(JsValue::from);
        Array::from_iter(outputs)
    }

    #[wasm_bindgen(setter = outputs)]
    pub fn set_outputs_from_js_array(&mut self, js_value: &JsValue) {
        let outputs = Array::from(js_value)
            .iter()
            .map(|js_value| {
                ref_from_abi!(TransactionOutput, &js_value).unwrap_or_else(|err| panic!("invalid transaction output: {err}"))
            })
            .collect::<Vec<_>>();
        self.inner().outputs = outputs;
    }

    #[wasm_bindgen(getter, js_name = version)]
    pub fn get_version(&self) -> u16 {
        self.inner().version
    }

    #[wasm_bindgen(setter, js_name = version)]
    pub fn set_version(&self, v: u16) {
        self.inner().version = v;
    }

    #[wasm_bindgen(getter, js_name = lock_time)]
    pub fn get_lock_time(&self) -> u64 {
        self.inner().lock_time
    }

    #[wasm_bindgen(setter, js_name = lock_time)]
    pub fn set_lock_time(&self, v: u64) {
        self.inner().lock_time = v;
    }

    #[wasm_bindgen(getter, js_name = gas)]
    pub fn get_gas(&self) -> u64 {
        self.inner().lock_time
    }

    #[wasm_bindgen(setter, js_name = gas)]
    pub fn set_gas(&self, v: u64) {
        self.inner().lock_time = v;
    }

    #[wasm_bindgen(getter = subnetworkId)]
    pub fn get_subnetwork_id_as_hex(&self) -> String {
        self.inner().subnetwork_id.to_hex()
    }

    #[wasm_bindgen(setter = subnetworkId)]
    pub fn set_subnetwork_id_from_js_value(&mut self, js_value: JsValue) {
        let subnetwork_id = js_value.try_as_vec_u8().unwrap_or_else(|err| panic!("subnetwork id error: {err}"));
        self.inner().subnetwork_id = subnetwork_id.as_slice().try_into().unwrap_or_else(|err| panic!("subnetwork id error: {err}"));
    }

    #[wasm_bindgen(getter = payload)]
    pub fn get_payload_as_hex_string(&self) -> String {
        self.inner().payload.to_hex()
    }

    #[wasm_bindgen(setter = payload)]
    pub fn set_payload_from_js_value(&mut self, js_value: JsValue) {
        self.inner.lock().unwrap().payload = js_value.try_as_vec_u8().unwrap_or_else(|err| panic!("payload value error: {err}"));
    }
}

impl TryFrom<JsValue> for Transaction {
    type Error = Error;
    fn try_from(js_value: JsValue) -> std::result::Result<Self, Self::Error> {
        if js_value.is_object() {
            let object = Object::from(js_value);
            let version = object.get_u16("version")?;
            workflow_log::log_trace!("JsValue->Transaction: version: {version:?}");
            let lock_time = object.get_u64("lockTime")?;
            let gas = object.get_u64("gas")?;
            let payload = object.get_vec_u8("payload")?;
            let subnetwork_id = object.get_vec_u8("subnetworkId")?;
            if subnetwork_id.len() != subnets::SUBNETWORK_ID_SIZE {
                return Err(Error::Custom("subnetworkId must be 20 bytes long".into()));
            }
            let subnetwork_id: SubnetworkId =
                subnetwork_id.as_slice().try_into().map_err(|err| Error::Custom(format!("`subnetworkId` property error: `{err}`")))?;
            workflow_log::log_trace!("JsValue->Transaction: subnetwork_id: {subnetwork_id:?}");
            let inputs = object
                .get_vec("inputs")?
                .into_iter()
                .map(|jsv| jsv.try_into())
                .collect::<std::result::Result<Vec<TransactionInput>, Error>>()?;
            workflow_log::log_trace!("JsValue->Transaction: inputs.len(): {:?}", inputs.len());
            let jsv_outputs = object.get("outputs")?;
            let outputs: Vec<TransactionOutput> = if !jsv_outputs.is_array() {
                let outputs: PaymentOutputs = jsv_outputs.try_into()?;
                outputs.into()
            } else {
                object
                    .get_vec("outputs")?
                    .into_iter()
                    .map(|jsv| {
                        workflow_log::log_trace!("JsValue->Transaction: output : {jsv:?}");
                        jsv.try_into()
                    })
                    .collect::<std::result::Result<Vec<TransactionOutput>, Error>>()?
            };
            workflow_log::log_trace!("JsValue->Transaction: outputs: {outputs:?}");
            Transaction::new(version, inputs, outputs, lock_time, subnetwork_id, gas, payload)
        } else {
            Err("Transaction must be an object".into())
        }
    }
}

impl TryFrom<cctx::Transaction> for Transaction {
    type Error = Error;
    fn try_from(tx: cctx::Transaction) -> std::result::Result<Self, Self::Error> {
        let id = tx.id();
        let inputs: Vec<TransactionInput> =
            tx.inputs.into_iter().map(|input| input.try_into()).collect::<std::result::Result<Vec<TransactionInput>, Error>>()?;
        let outputs: Vec<TransactionOutput> =
            tx.outputs.into_iter().map(|output| output.try_into()).collect::<std::result::Result<Vec<TransactionOutput>, Error>>()?;
        Ok(Self::new_with_inner(TransactionInner {
            version: tx.version,
            inputs,
            outputs,
            lock_time: tx.lock_time,
            gas: tx.gas,
            payload: tx.payload,
            subnetwork_id: tx.subnetwork_id,
            id, // : tx.id(),
        }))
    }
}

impl TryFrom<&Transaction> for cctx::Transaction {
    type Error = Error;
    fn try_from(tx: &Transaction) -> std::result::Result<Self, Self::Error> {
        let inner = tx.inner();
        let inputs: Vec<cctx::TransactionInput> = inner
            .inputs
            .clone()
            .into_iter()
            .map(|input| input.try_into())
            .collect::<std::result::Result<Vec<cctx::TransactionInput>, Error>>()?;
        let outputs: Vec<cctx::TransactionOutput> = inner
            .outputs
            .clone()
            .into_iter()
            .map(|output| output.try_into())
            .collect::<std::result::Result<Vec<cctx::TransactionOutput>, Error>>()?;
        Ok(cctx::Transaction::new(
            inner.version,
            inputs,
            outputs,
            inner.lock_time,
            inner.subnetwork_id.clone(),
            inner.gas,
            inner.payload.clone(),
        ))
    }
}
