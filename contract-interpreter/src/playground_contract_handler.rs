use std::collections::BTreeMap;
use guardian_common::custom_types::*;


// Create generic contract from content of Aqua-Chain if possible
pub fn generic_contract<'a>(
    contract_content_main: &'a Vec<&'a str>,
    contract_content_transclusion_hash: &'a str,
) -> Option<Contract<'a>> {

    let params_vec = &contract_content_main[1..];

    // Extract parameters from string defined by key "main"
    let Some(params): Option<BTreeMap<_, _>> = params_vec
        .into_iter()
        .map(|s| {
            let s = s.strip_prefix("|")?;
            s.split_once("=").to_owned()
        })
        .collect()
    else {
        println!("Error while converting params_vec into params");
        return None;
    };

    // check if tranclusion hash is in the correct format defined by Hash 
    match contract_content_transclusion_hash.parse(){
        Ok(transclusion_hash) => {
            // Construct generic contract using Name, Parameters and Transclusion verification hash
            return Some(Contract {
            name: contract_content_main[0],
            parameters: params,
            transclusion_verification_hash: transclusion_hash,
            })
        },
        Err(_) =>{
            println!("Failed to parse verification hash from transcluion");
            return  None;
        },
    };

    
}

// Detects specific contract in the generic contract
pub fn detect_contract<'a>(contract: &'a Contract<'a>) -> Option<ContractType<'a>> {
    // If true, detects DAA 
    if contract.transclusion_verification_hash == "109d3582839c6b2d616fbb96057f6665a68bbe6ec755358822b4329f51e279a9c9fb1b4b466fc8ca3e9c335aacbb759e5977e6a95acfd669e2c40033c08ea40f".parse().unwrap() {
        Some(ContractType::DataAccessAgreement(handle_daa(&contract.parameters)?))
    }
    else if contract.transclusion_verification_hash == "e9bb93fc861e2825d2c5866c496e04f209dd3f25615a046f114b979c1869a138300e4890cf9bb246ab7ea6dd7eb3a57cc2b975ec8790fb4eb6392484f022bea6".parse().unwrap() {
        Some(ContractType::IdentityClaim(handle_ic(&contract.parameters)?))
    }
    else {
        None
    }
}

//  Constructs DAA
//  Context: 
pub fn handle_daa<'a>(contract_parameters : &BTreeMap<&'a str, &'a str>) -> Option<DataAccessAgreement <'a>> {

    // Extract file identifier (Name of the page)
    let Some(file_identifier) = contract_parameters.get("file_identifier")
    else {
        println!("file_identifier not found!");
        return None
    };

    // Extract transclusion hash of the file that is to be shared
    //let Some()

    // Extract account of the receiver
    let Some(account_receiver) = contract_parameters.get("account_receiver")
    else {
        println!("account_receiver not found!");
        return None
    };
    
    match account_receiver.parse(){
        Ok(account_receiver) => {
        // Extract text message, if it was specified
        let free_text = contract_parameters.get("free_text");
        let daa:DataAccessAgreement =  DataAccessAgreement{
        file_identifier,
        account_receiver,
        free_text:free_text.copied(),
        };
        return Some(daa);
        },
        Err(_) => {
            println!("Failed to parse receiver account");
            return  None;
        },
    };

    }
    
//  Constructs IC
//  Context: 
pub fn handle_ic<'a>(contract_parameters : &BTreeMap<&'a str, &'a str>) -> Option<IdentityClaim>{

    // extract account of the guardian
    let Some(account_guardian) = contract_parameters.get("account_guardian")
    else{
        println!("account_guardian not found!");
        return None
    };

    // extract account of guardian's authoritative user
    let Some(account_auth_user) = contract_parameters.get("account_auth_user")
    else{
        println!("account_auth_user not found!");
        return None;
    };

    // check if account of the guardian is in the correct format defined by PublicKey
    match account_guardian.parse(){
        Ok(account_guardian) =>{
            // check if account of guardian's authoritative user is also in the correct format defined by PublicKey
            match account_auth_user.parse(){
                Ok(account_auth_user) => {
                    let ic:IdentityClaim = IdentityClaim{
                        account_guardian,
                        account_auth_user,
                    };
                    return Some(ic);
                },
                Err(_) => {
                    println!("failed to parse account of guardian's autoritative user");
                    return None;
                },
            }
        },
        Err(_) => {
            println!("failed to parse account of the guardian");
            return None;
        },
    }

}

// todo 
//pub fn handle_handshake(contract_parameters : &BTreeMap<&'a str, &'a str>) {}


//////////////////////////////////////////////////////////////////////////
/// Structures

pub struct SharedPage<'a>{
    file_identifier: &'a str,
    transclusion_hash: Hash,
}
pub struct DataAccessAgreement<'a> {
    shared_page: SharedPage<'a>,
    account_receiver: PublicKey,
    free_text: Option<&'a str>,
}

pub struct IdentityClaim {
    account_guardian: PublicKey,
    account_auth_user : PublicKey,
}

pub struct Contract<'a> {
    pub name: &'a str,
    pub parameters: BTreeMap<&'a str, &'a str>,
    pub transclusion_verification_hash: Hash,
}

pub struct Handshake{
    pub account_sender: PublicKey,
    pub certificate_guardian: GuardianCertificate,
}
// todo
pub struct GuardianCertificate{
    account_guardian: PublicKey,
    //?? guardian_signature: Signature,
}

pub enum ContractType <'b> {
    DataAccessAgreement(DataAccessAgreement<'b>),
    IdentityClaim (IdentityClaim),
    Handshake (Handshake),
}

// // Allowed contracts defined by the translcusion verification hash for the corresponding contract template
// pub enum AllowedContracts<'b> {
//     // todo
//     AllowedContractsMap(Map<&'b str, Hash>),
// }
