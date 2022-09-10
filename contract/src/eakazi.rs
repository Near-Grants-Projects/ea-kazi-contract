use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, LazyOption, LookupSet};
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, Gas, Promise, ext_contract, require};
use near_contract_standards::non_fungible_token::{Token, TokenId};

pub const CERTIFICATE_CONTRACT: &str = "certificate.eakazi.testnet";
pub const XCC_GAS: Gas = Gas(20000000000000);


#[derive(BorshDeserialize, BorshSerialize)]
pub struct Course {
    id: U128,
    name: String,
    description: String,
    course_key: String,
    skills: Vec<U128>
}


impl Course {
    pub fn new(id_: U128, name_: String, description_: String, course_key_: String, skills_: Vec<U128>) -> Self {
        Self {
            id: id_,
            name: name_,
            description: description_,
            course_key: course_key_,
            skills: skills_
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct CertificateToken {
    token_id: TokenId,
    skills: LookupSet<U128>,
    course: U128,
    issuer: AccountId
}

impl CertificateToken {
    pub fn new(nft: Token, skills: Vec<U128>, course: U128, trainer: AccountId) -> Self {
        let mut token = Self {
            token_id: nft.token_id,
            skills: LookupSet::new(b"s"),
            course: course,
            issuer: trainer
        };

        for skill in skills {
            token.skills.insert(&skill);
        }

        token
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Job {
    id: U128,
    name: String,
    description: String,
    status: u8,
    skills: Vec<U128>,
    job_owner: AccountId,
    wage: U128,
    number_of_roles: u128,
    number_of_roles_taken: u128
}

impl Job {
    pub fn new(id_: U128, name_: String, description_: String, skills_: Vec<U128>, job_owner_: AccountId, wage_: U128, number_of_roles_: u128) -> Self {
        Self {
            id: id_,
            name: name_,
            description: description_,
            skills: skills_,
            job_owner: job_owner_,
            wage: wage_,
            number_of_roles: number_of_roles_,
            status: 0,
            number_of_roles_taken: 0
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct JobApplication {
    applicant: AccountId,
    job_id: U128,
    status: u8,
    start_date: Option<u64>,
    trainers_to_pay: Vec<AccountId>
}

impl JobApplication {
    pub fn new(job_id_: U128, trainers: Vec<AccountId>, applicant_: AccountId) -> Self {
        Self {
            applicant: applicant_,
            job_id: job_id_,
            trainers_to_pay: trainers,
            start_date: None,
            status: 0
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
    courses: LookupMap<U128, Course>,
    jobs: LookupMap<U128, Job>,
    job_applications: LookupMap<U128, LookupMap<AccountId, JobApplication>>,
    course_trainer: LookupMap<U128, AccountId>,
    enrolments: LookupMap<U128, Vec<AccountId>>,
    certificates: LookupMap<AccountId, Vec<CertificateToken>>,
    // user_skills: LookupMap<AccountId, LookupSet<U128>>,
    wages_in_escrow: LookupMap<AccountId, U128>
    
}

impl Default for EAKazi {
    fn default() -> Self {
        Self {
            courses: LookupMap::new(b"c"),
            jobs: LookupMap::new(b"j"),
            job_applications: LookupMap::new(b"m"),
            course_trainer: LookupMap::new(b"k"),
            enrolments: LookupMap::new(b"e"),
            certificates: LookupMap::new(b"t"),
            wages_in_escrow: LookupMap::new(b"w")
        }
    }
}


#[near_bindgen]
impl EAKazi {

    // fn to_json_event_string(&self) -> String {
    //     format!("EVENT_JSON:{}", self.to_json_string())
    // }

    // this function takes owner as parameter as it will only be called when course is verified by admin.
    pub fn create_course(&mut self, owner: AccountId, course_id: U128, name_: String, description_: String, course_key_: String, skills_: Vec<U128>) {
        require!(!self.course_exists(&course_id), "This course is already created");
        let course = Course::new(course_id.clone(), name_, description_, course_key_, skills_);
        self.courses.insert(&course_id, &course);
        self.course_trainer.insert(&course_id, &owner);
    }

    // pub fn activate_course(&mut self)

    pub fn enroll_for_course(&mut self, course_id: U128) {
        //require!(self.get_course_by_id(&course_id).unwrap(), "This course has not been created");
        let trainee = env::signer_account_id();
        let mut registered_trainees = self.enrolments.get(&course_id).unwrap_or_default();
        registered_trainees.push(trainee);
        self.enrolments.insert(&course_id, &registered_trainees);
    }

    pub fn mint_certificate_to_trainee(&mut self, course_id: U128, trainee_id: AccountId, certificate_url: String, issue_date: String) {
        let course_ = self.get_course_by_id(&course_id).unwrap();

        // require!(course_  false, "Course with requested id does not exist");
        ext_certificate_contract::ext(AccountId::new_unchecked(CERTIFICATE_CONTRACT.to_string()))
            .nft_mint(
                trainee_id.clone(),
                self.course_trainer.get(&course_id).unwrap(),
                course_.name.clone(),
                certificate_url,
                issue_date)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(XCC_GAS)
                    .on_mint_certificate_callback(trainee_id, course_.skills.clone(), U128::from(course_.id), self.course_trainer.get(&course_id).unwrap())
            );

        
    }

    // for jobs exceeding 12 months, 12 month wage would be initially deducted
    #[payable]
    pub fn create_job(&mut self, job_id_: U128, job_owner: AccountId, name: String, description_: String, skills_: Vec<U128>, wage: U128, number_of_roles: u128, duration: u128) {

        let wage_to_pay = wage.0 * number_of_roles * duration;
        require!(env::attached_deposit() >= wage_to_pay, "Attach wage to be paid to escrow first");
        let job_owner = env::signer_account_id();
        let job = Job::new(job_id_, name, description_, skills_, job_owner.clone(), wage, number_of_roles);
        self.jobs.insert(&job_id_, &job);
        let previous_wages_in_escrow = self.wages_in_escrow.get(&job_owner).unwrap_or(U128::from(0));
        let increment_wage = U128::from(wage_to_pay + previous_wages_in_escrow.0);
        self.wages_in_escrow.insert(&job_owner, &increment_wage);
    }

    pub fn apply_to_job(&mut self, job_id: U128) {
        require!(self.job_exists(&job_id), "Job with provided id does not exist");

        let applicant = env::signer_account_id();
        let trainers_for_job = self.user_has_skills_for_job(&applicant, &job_id).unwrap_or_default();
        
        require!(trainers_for_job.len() > 0, "You are not skilled enough for the job");

        let mut trainers = Vec::new();

        for trainer in trainers_for_job {
            trainers.push(trainer.clone());
        }

        let application = JobApplication::new(job_id.clone(), trainers, applicant.clone());
        let mut apps = self.job_applications.get(&job_id).unwrap();
        apps.insert(&applicant, &application);

        self.job_applications.insert(&job_id, &apps);

    }

    pub fn confirm_job_emloyment(&mut self, job_id: U128, applicant: AccountId) {
        require!(self.job_exists(&job_id), "Job with provided id does not exist");
        require!(env::signer_account_id() == self.get_job_by_id(&job_id).unwrap().job_owner, "Only job owner can confirm employment");

        let mut apps = self.job_applications.get(&job_id).unwrap();
        require!(&apps.contains_key(&applicant), "The applicant has not applied for this job");
        
        let mut job_app = apps.get(&applicant).unwrap();

        job_app.status = 1;
        job_app.start_date = Some(env::block_timestamp());
        
        apps.insert(&applicant, &job_app);
        self.job_applications.insert(&job_id, &apps);
    }


    pub fn end_job_employment(&mut self, job_id: U128, applicant: AccountId) {
        require!(self.job_exists(&job_id), "Job with provided id does not exist");
        require!(env::signer_account_id() == self.get_job_by_id(&job_id).unwrap().job_owner, "Only job owner can confirm employment");

        let mut apps = self.job_applications.get(&job_id).unwrap();
        require!(&apps.contains_key(&applicant), "The applicant has not applied for this job");
        
        let mut job_app = apps.get(&applicant).unwrap();

        job_app.status = 2;
        
        apps.insert(&applicant, &job_app);
        self.job_applications.insert(&job_id, &apps);
    }

    pub fn pay_wage(&mut self, job_id: U128, applicant: AccountId) {
        require!(self.job_exists(&job_id), "Job with provided id does not exist");
        let job = self.get_job_by_id(&job_id).unwrap();
        let owner = job.job_owner;
        require!(env::signer_account_id() == owner.clone(), "Only job owner can confirm wage payment");

        let mut wage_in_excrow = self.wages_in_escrow.get(&owner).unwrap_or(U128::from(0));
        let wage_for_job = job.wage;

        let application = self.get_job_aplication(&job_id, &applicant).unwrap();
        let trainers = application.trainers_to_pay;

        let wage_to_applicant = (wage_for_job.0 * 9)/10;
        let wage_to_trainer = (wage_for_job.0 - wage_to_applicant)/u128::try_from(trainers.len()).unwrap_or(1);

        wage_in_excrow = U128::from(wage_in_excrow.0 - wage_for_job.0);

        Promise::new(applicant).transfer(wage_to_applicant)
            .then(Self::ext(env::current_account_id())
                .with_static_gas(XCC_GAS)
                .on_pay_wage_callback(owner, wage_in_excrow));

        for trainer in trainers {
            Promise::new(trainer).transfer(wage_to_trainer);
        }
    }



    // will check if user has the skills for the job and return the trainers who taught the skill
    #[private]
    pub fn user_has_skills_for_job(&self, user: &AccountId, job_id: &U128) -> Option<Vec<AccountId>> {
        let job = self.get_job_by_id(job_id).unwrap();
        let job_skills = &job.skills;
        let user_certs = self.certificates.get(&user).unwrap_or_default();

        // number of skills user has for job
        let mut number_of_matches = 0;

        // trainers that would be paid from the job
        let mut trainers_for_skills = Vec::new();

        for skill in job_skills.clone() {
            let mut current_skill_check = 0;

            for cert in &user_certs {
                if cert.skills.contains(&skill) {
                    trainers_for_skills.push(&cert.issuer);
                    if current_skill_check > 1 {
                        continue
                    }
                    number_of_matches = number_of_matches + 1;
                    current_skill_check = current_skill_check + 1;
                }
            }
        }

        let mut trainers = Vec::new();
        for trainer in trainers_for_skills {
            trainers.push(trainer.clone());
        }

        let percentage_match = number_of_matches * 100/u128::try_from(job_skills.len()).unwrap_or(1);
        if percentage_match > 50 {
            return Some(trainers);
        }

        None
    }

    #[private]
    pub fn course_exists(&self, course_id_: &U128) -> bool {
        self.courses.contains_key(course_id_)
    }

    #[private]
    pub fn job_exists(&self, job_id: &U128) -> bool {
        self.jobs.contains_key(job_id)
    }

    #[private]
    pub fn is_user_enrolled(&self, course_id_: &U128, user_id: &AccountId) -> bool {
        for trainee in self.enrolments.get(course_id_).unwrap() {
            if &trainee == user_id {
                return true;
            }
        }
        false
    }

    #[private]
    #[result_serializer(borsh)]
    pub fn get_course_by_id(&self, course_id_: &U128) -> Option<Course> {
        self.courses.get(&course_id_)
    }

    #[private]
    #[result_serializer(borsh)]
    pub fn get_job_by_id(&self, job_id_: &U128) -> Option<Job> {
        self.jobs.get(&job_id_)
    }

    #[private]
    #[result_serializer(borsh)]
    pub fn get_job_aplication(&self, job_id_: &U128, applicant: &AccountId) -> Option<JobApplication> {
        self.job_applications.get(job_id_).unwrap().get(applicant)
    }

    #[private]
    pub fn on_mint_certificate_callback(&mut self, owner_id: AccountId, skills: Vec<U128>, course_id: U128, trainer: AccountId, #[callback_unwrap] token: Token) {
        let certificate = CertificateToken::new(token.clone(), skills, course_id, trainer);
        let mut user_certs = self.certificates.get(&owner_id).unwrap_or_default();
        user_certs.push(certificate);
        self.certificates.insert(&owner_id, &user_certs);
    }

    #[private]
    pub fn on_pay_wage_callback(&mut self, job_owner: AccountId, new_escrow: U128) {
        self.wages_in_escrow.insert(&job_owner, &new_escrow); 
    }
}