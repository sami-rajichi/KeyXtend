//! A count's model and device as tags: any other file, a table row by its id, or a cloud service by its id and its
//! listed model's, if kept; the processor, a card by its id, or a cloud service.

use kx_module_api::ports::{BlobName, CloudAsk};

use super::card;
use crate::counts::{Device, Model};
use crate::models::id_ok;
use crate::store::codec::{Bad, Reader, Writer};

/// A model's tag, then a row's id, or a service's id and its listed model's if kept.
const OTHER: u8 = 0;
const ROW: u8 = 1;
const CLOUD: u8 = 2;
/// A device's tag: the processor, a card followed by its id, then a cloud service.
const CPU: u8 = 0;
const CARD: u8 = 1;
const NET: u8 = 2;

pub(super) fn put_model(w: &mut Writer, model: &Model) {
    match model {
        Model::Other => w.u8(OTHER),
        Model::Row(id) => {
            w.u8(ROW);
            w.text(id);
        }
        Model::Cloud { service, model } => {
            w.u8(CLOUD);
            w.text(service);
            w.flag(model.is_some());
            if let Some(model) = model {
                w.text(model);
            }
        }
    }
}

/// A count's model; a service id that names no blob, or a model id no request could carry, is `Damaged`.
pub(super) fn model(r: &mut Reader<'_>) -> Result<Model, Bad> {
    match r.u8()? {
        OTHER => Ok(Model::Other),
        ROW => Ok(Model::Row(r.text().map(str::to_owned).and_then(row_id)?)),
        CLOUD => {
            let service = r.text().map(str::to_owned).and_then(service_id)?;
            let model = if r.flag()? {
                Some(r.text().map(str::to_owned).and_then(cloud_model)?)
            } else {
                None
            };
            Ok(Model::Cloud { service, model })
        }
        _ => Err(Bad::Damaged),
    }
}

pub(super) fn put_device(w: &mut Writer, device: &Device) {
    match device {
        Device::Cpu => w.u8(CPU),
        Device::Card(card) => {
            w.u8(CARD);
            w.text(card.as_str());
        }
        Device::Cloud => w.u8(NET),
    }
}

pub(super) fn device(r: &mut Reader<'_>) -> Result<Device, Bad> {
    match r.u8()? {
        CPU => Ok(Device::Cpu),
        CARD => card(r).map(Device::Card),
        NET => Ok(Device::Cloud),
        _ => Err(Bad::Damaged),
    }
}

/// `id` when it is a row id as the table writes them.
fn row_id(id: String) -> Result<String, Bad> {
    if id_ok(&id) {
        Ok(id)
    } else {
        Err(Bad::Damaged)
    }
}

/// `id` when it names a blob, as each service's id in the table does.
fn service_id(id: String) -> Result<String, Bad> {
    if BlobName::new(&id).is_ok() {
        Ok(id)
    } else {
        Err(Bad::Damaged)
    }
}

/// `id` when a request could carry it as a model's id.
fn cloud_model(id: String) -> Result<String, Bad> {
    if CloudAsk::model_fits(&id) {
        Ok(id)
    } else {
        Err(Bad::Damaged)
    }
}
