from typing import Optional
import uvicorn

from fastapi import FastAPI, Body, Depends, HTTPException,  File, UploadFile
from fastapi.middleware.cors import CORSMiddleware
from config import *
from pydantic import BaseModel
import json

# import data
from hotaSolana.hotaSolanaDataBase import *
from hotaSolana.hotaSolanaData import *
from hotaSolana.bs58 import bs58

from baseAPI import *

description = """
hotaSolana API helps you do awesome stuff. 🚀
"""

app = FastAPI(title="Solana API",
              description=description,
              summary="This is a Solana API",
              version="v2.0",
              contact={
                  "name": "Hotamago Master",
                  "url": "https://www.linkedin.com/in/hotamago/",
              })

origins = ["*"]

app.add_middleware(
    CORSMiddleware,
    allow_origins=origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Solana Client
client = HotaSolanaRPC(programId, False, "devnet")

# Solana instruction data
@BaseStructClass
class Coordinates:
    latitude=HotaFloat64()
    longitude=HotaFloat64()

# Solana account data
@BaseStructClass
class Depot:
    owner=HotaPublicKey()
    name=HotaStringUTF16(32)
    info=HotaStringUTF16(256)
    coordinates=Coordinates()
    time_created=HotaIntX(8)

@BaseStructClass
class Items:
    owner=HotaPublicKey()
    depot=HotaPublicKey()
    name=HotaStringUTF16(32)
    info=HotaStringUTF16(256)
    coordinates=Coordinates()
    cur_time=HotaIntX(8)
    time_created=HotaIntX(8)

# Solana instruction
@BaseInstructionDataClass("init_depot")
class InitDepotInstruction:
    name=HotaStringUTF16(32)
    info=HotaStringUTF16(256)
    coordinates=Coordinates()
    seed_random=HotaUintX(16)

@BaseInstructionDataClass("init_items")
class InitItemsInstruction:
    name=HotaStringUTF16(32)
    info=HotaStringUTF16(256)
    coordinates=Coordinates()
    seed_random=HotaUintX(16)

@BaseInstructionDataClass("update_items")
class UpdateItemsInstruction:
    coordinates=Coordinates()

@BaseInstructionDataClass("transfer_items")
class TransferItemsInstruction:
    pass

##### Router

class CoordinatesModel(BaseModel):
    latitude: float
    longitude: float

# init_depot
class InitDepotModel(BaseModel):
    owner_private_key: str
    name: str
    info: str
    coordinates: CoordinatesModel

@app.post("/init-depot")
async def init_depot(
    initDepotModel: InitDepotModel,
):
    def fun():
        owner_keypair = makeKeyPair(initDepotModel.owner_private_key)

        instruction_data = InitDepotInstruction()
        instruction_data.get("coordinates").get("latitude").object2struct(initDepotModel.coordinates.latitude)
        instruction_data.get("coordinates").get("longitude").object2struct(initDepotModel.coordinates.longitude)
        instruction_data.get("name").object2struct(initDepotModel.name)
        instruction_data.get("info").object2struct(initDepotModel.info)
        instruction_data.get("seed_random").random()

        print(instruction_data.struct2object())

        depot_pubkey = findProgramAddress(createBytesFromArrayBytes(
            owner_keypair.public_key.byte_value,
            "depot".encode("utf-8"),
            bytes(instruction_data.get("seed_random").serialize()),
        ), client.program_id)

        instruction_address = client.send_transaction(
            instruction_data,
            [
                makePublicKey(sysvar_clock),
                makeKeyPair(payerPrivateKey).public_key,
                owner_keypair.public_key,
                depot_pubkey,
                makePublicKey(sysvar_rent),
                makePublicKey(system_program),
            ],
            [
                makeKeyPair(payerPrivateKey),
                owner_keypair,
            ],
            fee_payer=makeKeyPair(payerPrivateKey).public_key
        )

        return {
            "instruction_address": instruction_address,
            "public_key": bs58.encode(depot_pubkey.byte_value),
        }

    return make_response_auto_catch(fun)

# init_items
class InitItemsModel(BaseModel):
    owner_private_key: str
    depot_owner_private_key: str
    depot_public_key: str
    name: str
    info: str
    coordinates: CoordinatesModel

@app.post("/init-items")
async def init_items(
    initItemsModel: InitItemsModel,
):
    def fun():
        owner_keypair = makeKeyPair(initItemsModel.owner_private_key)
        depot_owner_keypair = makeKeyPair(initItemsModel.depot_owner_private_key)
        depot_pubkey = PublicKey(initItemsModel.depot_public_key)

        instruction_data = InitItemsInstruction()
        instruction_data.get("coordinates").get("latitude").object2struct(initItemsModel.coordinates.latitude)
        instruction_data.get("coordinates").get("longitude").object2struct(initItemsModel.coordinates.longitude)
        instruction_data.get("name").object2struct(initItemsModel.name)
        instruction_data.get("info").object2struct(initItemsModel.info)
        instruction_data.get("seed_random").random()

        items_pubkey = findProgramAddress(createBytesFromArrayBytes(
            owner_keypair.public_key.byte_value,
            depot_owner_keypair.public_key.byte_value,
            "items".encode("utf-8"),
            bytes(instruction_data.get("seed_random").serialize()),
        ), client.program_id)

        instruction_address = client.send_transaction(
            instruction_data,
            [
                makePublicKey(sysvar_clock),
                makeKeyPair(payerPrivateKey).public_key,
                owner_keypair.public_key,
                depot_owner_keypair.public_key,
                depot_pubkey,
                items_pubkey,
                makePublicKey(sysvar_rent),
                makePublicKey(system_program),
            ],
            [
                makeKeyPair(payerPrivateKey),
                owner_keypair,
                depot_owner_keypair,
            ],
            fee_payer=makeKeyPair(payerPrivateKey).public_key
        )

        return {
            "instruction_address": instruction_address,
            "public_key": bs58.encode(items_pubkey.byte_value),
        }

    return make_response_auto_catch(fun)

# update_items
class UpdateItemsModel(BaseModel):
    owner_private_key: str
    items_public_key: str
    coordinates: CoordinatesModel

@app.post("/update-items")
async def update_items(
    updateItemsModel: UpdateItemsModel,
):
    def fun():
        owner_keypair = makeKeyPair(updateItemsModel.owner_private_key)
        items_pubkey = PublicKey(updateItemsModel.items_public_key)

        instruction_data = UpdateItemsInstruction()
        instruction_data.get("coordinates").get("latitude").object2struct(updateItemsModel.coordinates.latitude)
        instruction_data.get("coordinates").get("longitude").object2struct(updateItemsModel.coordinates.longitude)

        instruction_address = client.send_transaction(
            instruction_data,
            [
                makePublicKey(sysvar_clock),
                makeKeyPair(payerPrivateKey).public_key,
                owner_keypair.public_key,
                items_pubkey,
                makePublicKey(sysvar_rent),
                makePublicKey(system_program),
            ],
            [
                makeKeyPair(payerPrivateKey),
                owner_keypair,
            ],
            fee_payer=makeKeyPair(payerPrivateKey).public_key
        )

        return {
            "instruction_address": instruction_address,
            "public_key": bs58.encode(items_pubkey.byte_value),
        }

    return make_response_auto_catch(fun)

# transfer_items
class TransferItemsModel(BaseModel):
    old_depot_owner_private_key: str
    old_depot_public_key: str
    new_depot_owner_private_key: str
    new_depot_public_key: str
    items_public_key: str

@app.post("/transfer-items")
async def transfer_items(
    transferItemsModel: TransferItemsModel,
):
    def fun():
        old_depot_owner_keypair = makeKeyPair(transferItemsModel.old_depot_owner_private_key)
        new_depot_owner_keypair = makeKeyPair(transferItemsModel.new_depot_owner_private_key)
        old_depot_pubkey = PublicKey(transferItemsModel.old_depot_public_key)
        new_depot_pubkey = PublicKey(transferItemsModel.new_depot_public_key)

        items_pubkey = PublicKey(transferItemsModel.items_public_key)

        instruction_data = TransferItemsInstruction()

        instruction_address = client.send_transaction(
            instruction_data,
            [
                makePublicKey(sysvar_clock),
                makeKeyPair(payerPrivateKey).public_key,
                old_depot_owner_keypair.public_key,
                new_depot_owner_keypair.public_key,
                old_depot_pubkey,
                new_depot_pubkey,
                items_pubkey,
                makePublicKey(sysvar_rent),
                makePublicKey(system_program),
            ],
            [
                makeKeyPair(payerPrivateKey),
                old_depot_owner_keypair,
                new_depot_owner_keypair,
            ],
            fee_payer=makeKeyPair(payerPrivateKey).public_key
        )

        return {
            "instruction_address": instruction_address,
            "public_key": bs58.encode(items_pubkey.byte_value),
        }

    return make_response_auto_catch(fun)

#### Common function1
@app.post("/convert-keypair-to-private-key")
async def convert_keypair_to_private_key(file: UploadFile):
    # Bytes to string
    result = file.file.read()
    keypair_json = json.loads(result)
    keypair_bytes = bytes(keypair_json)
    return {
        "public_key": bs58.encode(keypair_bytes[32:]),
        "private_key": bs58.encode(keypair_bytes),
    }

@app.get("/get-depot-info")
async def get_depot_info(public_key: str):
    return make_response_auto_catch(lambda: client.get_account_info(PublicKey(public_key)))

@app.get("/get-items-info")
async def get_items_info(public_key: str):
    return make_response_auto_catch(lambda: client.get_account_info(PublicKey(public_key)))

@app.get("/get-depot-data")
async def get_depot_data(public_key: str):
    def fun():
        res: dict = client.get_account_data(PublicKey(public_key), Depot, [8, 0])
        return res
    return make_response_auto_catch(fun)

@app.get("/get-items-data")
async def get_items_data(public_key: str):
    def fun():
        res: dict = client.get_account_data(PublicKey(public_key), Items, [8, 0])
        return res
    return make_response_auto_catch(fun)

@app.get("/get-balance")
async def get_balance(public_key: str):
    return make_response_auto_catch(client.get_balance(public_key))

@app.post("/airdrop")
async def airdrop(public_key: str, amount: int = 1):
    return make_response_auto_catch(client.drop_sol(public_key, amount))

# Run
if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=openPortAPI)
