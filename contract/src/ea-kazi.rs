use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, LazyOption};
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, Gas, Promise, ext_contract, require};
use near_contract_standards::non_fungible_token::{Token};

pub const CERTIFICATE_CONTRACT: &str = "certificate.eakazi.testnet";
pub const XCC_GAS: Gas = Gas(20000000000000);

#[derive(Default)]
pub struct Course {
    id: U128,
    name: String,
    description: String,
    status: u8,
    course_key: String,
}

impl Course {
    pub fn new(id_: U128, name_: String, description_: String, course_key_: String) -> Self {
        Self {
            id: id_,
            name: name,
            description: description_,
            course_key: course_key_,
            ..Default::default()
        }
    }
}

#[ext_contract(ext_certificate_contract)]
trait Certificate {
    fn nft_mint(
        &self, 
        token_owner_id: AccountId,
        trainer: AccountId,
        course_name: String,
        certificate_url: String,
        issue_date: String) -> Token;
}

// #[ext_contract(ext_self)]
// pub trait ExtSelf {
//     fn on_mint_certificate_callback(&mut self, ) -> AccountId;
// }

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct EAKazi {
    courses: Vec<Course>,
    course_trainer: LookupMap<U128, AccountId>,
    enrolments: LookupMap<U128, Vec<AccountId>>,
    certificates: LookupMap<AccountId, Vec<U128>>
}


#[near_bindgen]
impl EAKazi {

    pub fn create_course(&mut self, course_id: U128, name_: String, description_: String, course_key_: String) -> U128 {
        require!(!self.course_exists(course_key_), "This course is already created");
        let course = Course::new(course_id.clone(), name_, description_, course_key_);
        self.courses.push(course);
        let trainer_id = env::signer_account_id();
        let res = self.courses.insert(&course_id, trainer_id).unwrap();
        res
    }

    pub fn enroll_for_course(&mut self, course_id: U128) {
        require!(self.courses.contains_key(&course_id), "This course has not been created");
        let trainee = env::signer_account_id();
        let registered_trainees = self.enrolments.get(&course_id).unwrap();
        registered_trainees.push(&trainee);
        self.enrolments.insert(&course_id, &registered_trainees);
    }

    pub fn mint_certificate_to_trainee(&mut self, course_id: U128, trainee_id: AccountId, certificate_url: String, issue_date: String) {
        let course_ = self.get_course_by_id(course_id).ok_or("Course with provided id does not exist");

        ext_token_contract::ext(AccountId::new_unchecked(CERTIFICATE_CONTRACT.to_string()))
            .nft_mint(
                trainee_id,
                self.course_trainer.get(course_id).unwrap(),
                course_.name,
                certificate_url,
                issue_date)
            .then(
                owner = Self::ext(env::current_account_id())
                    .with_static_gas(XCC_GAS)
                    .on_mint_certificate_callback(trainee_id)
            );
    }

    #[private]
    pub fn course_exists(&self, course_key_: String) -> bool {
        for course in self.courses.iter() {
            if course.course_key = course_key_ {
                true
            }
        }
        false
    }

    #[private]
    pub fn is_user_enrolled(&self, course_id_: &U128, user_id: &AccountId) -> bool {
        for trainee in self.enrolments.get(course_id) {
            if trainer = user_id {
                true
            }
        }
        false
    }

    #[private]
    pub fn get_course_by_id(&self, course_id_: &U128) -> Option<Course> {
        for course in self.courses {
            if course.id = course_id_ {
                Some(course)
            }
        }
        None
    }

    #[private]
    pub fn on_mint_certificate_callback(&mut self, owner_id: AccountId, #[callback_unwrap] token: Token) {
        let user_certs = self.certificates.get(&owner_id).unwrap();
        user_certs.push(&token.token_id);
        self.certificates.insert(&owner_id, &user_certs);
    }
}