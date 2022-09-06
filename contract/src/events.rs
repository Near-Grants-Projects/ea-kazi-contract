use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::json_types::U128;
use near_sdk::AccountId;

pub struct CertificateMint {
    token_id: TokenId,
    trainee_id: AccountId,
    course_id: U128

}

pub struct CreateCourse {
    owner: AccountId,
    course_id: U128, 
    name: String, 
    description: String, 
    course_key: String
}

pub struct CourseEnroll {
    owner: AccountId,
    course_id: U128, 
    name: String, 
    description: String, 
    course_key: String
}

enum EventKind {
    CertificateMint(&[CertificateMint]),
    CreateCourse(&[CreateCourse]),
    CourseEnroll(&[CourseEnroll]),
    SetCourseBatch(&[SetCourseBatch]),
    RateTrainee(&[RateTrainee]),
    RateCourse(&[RateCourse]),

}