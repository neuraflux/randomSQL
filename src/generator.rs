#![allow(non_snake_case)]
#![allow(unused_assignments)]

use std::collections::HashMap;

use chrono::{NaiveDate, NaiveDateTime};
use fake::{
    faker::{address::en::*, company::en::*, internet::en::*, lorem::en::Word, name::raw::*, phone_number::en::*},
    locales::*,
    Fake,
    Faker,
    // More modules for mock data found at
    // https://docs.rs/fake/latest/fake/faker/index.html
};

use rand::{seq::SliceRandom, thread_rng, Rng};


pub(crate) fn check_compound_attribute(index: usize, attribute_checker: &Vec<String>) -> bool {
    /*
        * Checks if the compound attribute is valid
        * Compound attributes are of the form: (attribute1, attribute2, attribute3, ...)
        * Each attribute is separated by a semicolon

        :Parameters:
            - `index`: The index of the compound attribute in attribute_checker
            - `attribute_checker`: The vector of attributes to check

        :Returns:
            - `bool`: True if valid, false if invalid
    */

    // First, remove the first and last parenthesis of attribute_checker[2]
    let compound_attribute =
        attribute_checker[index][1..attribute_checker[index].len() - 1].to_string();

    // Split compound_attribute into a vector of attributes. Delimiter: ';'
    let compound_attributes = compound_attribute.split(";").collect::<Vec<&str>>();

    //New compound_attributes Vec -> [FIRST_NAME VARCHAR(20), MIDDLE_INITIAL CHAR(1), LAST_NAME VARCHAR(20)]
    //Iterate through compound_attributes, trim each element and check if each attribute is valid
    for compound_attribute in compound_attributes {
        let compound_attribute = compound_attribute.trim();
        let compound_attribute = compound_attribute.split(" ").collect::<Vec<&str>>();
        if compound_attribute.len() != 2 {
            println!("[!] Invalid Compound Attribute");
            return false;
        }
        if !check_data_type(compound_attribute[1].to_uppercase().as_str()) {
            println!("[!] Invalid Data Type");
            return false;
        }
    }
    true
}

pub(crate) fn check_data_type(attribute_type: &str) -> bool {
    /*
       * Checks if the data type is valid
       * Returns true if valid, false if invalid
       * Custom data types made for this program: EMAIL, GROUP, PHONE, SSN, STATE, ZIP, STREET-ADDRESS, FULL-ADDRESS, NAME, PASSWORD, USERNAME

       :Parameters:
           - `attribute_type`: The data type to check
       :Returns:
           - `bool`: True if valid, false if invalid

    */

    //Recreate valid types variable, but put all types in alphabetical order
    let valid_types = [
        "BIGINT",
        "BIT",
        "BOOLEAN",
        "BOX",
        "BYTEA",
        "CHAR",
        "CIDR",
        "CIRCLE",
        "CITY_SHORT",
        "CITY_US",
        "COMPANYNAME",
        "COMPOUND", // Not a real data type, used for compound attributes
        "COUNTRY",
        "DATE",
        "DECIMAL",
        "DOUBLE PRECISION",
        "EMAIL",
        "ENUM",
        "FLOAT4",
        "FLOAT8",
        "GROUP",
        "INDUSTRY",
        "INET",
        "INTEGER",
        "INTERVAL",
        "JSON",
        "JSONB",
        "LINE",
        "LSEG",
        "MACADDR",
        "MONEY",
        "NAME",
        "NUMERIC",
        "PASSWORD",
        "PATH",
        "PG_LSN",
        "PHONE",
        "POINT",
        "POLYGON",
        "PROFESSION",
        "REAL",
        "SERIAL",
        "SMALLINT",
        "SSN",
        "STATE",
        "STATE_US",
        "STREET_ADDRESS",
        "STREET_NAME_US",
        "TEXT",
        "TIME",
        "TIMESTAMP",
        "TSQUERY",
        "TSVECTOR",
        "TXID_SNAPSHOT",
        "USERNAME",
        "UUID",
        "VARCHAR",
        "XML",
        "ZIP_US",
    ];

    // Make sure the data type is valid
    valid_types
        .iter()
        .any(|&substring| attribute_type.starts_with(substring))
}

pub(crate) fn check_key_definition(key_def: &str) -> bool {
    /*
        * Checks if the key definition is properly defined
        * PK -> Primary Key
        * FK -> Foreign Key
        * AK -> Alternate (Unique) Key
        * PK/FK -> Primary Key and Foreign Key
        * AK/FK -> Alternate (Unique) Key and Foreign Key

        :parameters:
            - `key_def`: The key definition to check
        :returns:
            - `bool`: True if valid, false if invalid
    */

    let valid_keys = ["PK", "FK", "AK", "PK/FK", "AK/FK"];

    valid_keys.contains(&key_def)
}

pub(crate) fn check_pair(
    generated_pair_vector: &Vec<String>,
    previous_pairs: &Vec<Vec<String>>,
    table_attributes: &Vec<String>,
    uq_attributes: &HashMap<String, Vec<String>>,
    count: usize,
) -> (bool, Vec<String>) {
    /*
        * Recursively calls itself until it generates a valid composite key pair
        * All data used in pair generation comes from uq_attributes

        :parameters:
            - `generated_pair_vector`: The vector of generated pairs
            - `previous_pairs`: The vector of previously generated pairs
            - `table_attributes`: The vector of table attributes
            - `uq_attributes`: The hashmap of unique attributes
            - `count`: The number of attributes in the composite key

        :returns:
            - `bool`: True if valid, false if invalid
            - `Vec<String>`: The generated pair
    */
    let mut new_pair: Vec<String> = Vec::new();

    // First Time Running With Generated Pair
    let pair_changed = count > 0;

    let reverse_generated_pair: Vec<String> = generated_pair_vector
        .clone()
        .into_iter()
        .rev()
        .collect::<Vec<String>>();

    if previous_pairs.contains(&generated_pair_vector)
        || previous_pairs.contains(&reverse_generated_pair)
    {
        // Duplicate Pair Found
        // Must Generate New Data For Generated Pair
        for temp_attr in table_attributes.clone() {
            let temp_attr_list = temp_attr.trim().split(' ').collect::<Vec<&str>>();
            if temp_attr_list[0].to_uppercase() == "PK/FK" {
                let temp_attribute_name = temp_attr_list[3]
                    .split('(')
                    .nth(1)
                    .unwrap()
                    .replace(")", "");

                if let Some(history) = uq_attributes.get(&temp_attribute_name) {
                    loop {
                        if let Some(value) = history.choose(&mut thread_rng()) {
                            if !new_pair.contains(&value.to_string()) {
                                new_pair.push(value.to_string()); // Cast to &str
                                break;
                            }
                        }
                    }
                }
            }
        }
        return check_pair(
            &new_pair,
            previous_pairs,
            table_attributes,
            uq_attributes,
            count + 1,
        );
    }

    new_pair.extend_from_slice(&generated_pair_vector);
    (pair_changed, new_pair)
}


pub(crate) fn create_insert_statement(
    table_name: &str,
    table_attributes: &Vec<String>,
    statement_data: &HashMap<String, String>,
) -> String {
    /*
        * Creates the insert statement for the table
        * Example: INSERT INTO table_name VALUES (data1, data2, data3, ...)
        * The data is generated from the statement_data hashmap

        :parameters:
            - `table_name`: The name of the table
            - `table_attributes`: The vector of table attributes
            - `statement_data`: The hashmap of generated data for the table

        :returns:
            - `String`: The insert statement for the table

        TODO - Optimize this disgusting code
    */
    let mut insert_string = format!("INSERT INTO {} VALUES (", table_name);

    // Iterate over the generated data and add it to the INSERT statement
    // useless_key has no purpose and is only there to aid in iteration.
    for (index, attribute) in table_attributes.iter().enumerate() {
        let mut isCompound: bool = false;
        let compound_attribute;
        let mut compound_attribute_string = format!("(");
        let mut attribute_name: Vec<&str> = attribute.trim().split(" ").collect();

        if attribute_name.contains(&"COMPOUND") || attribute_name.contains(&"compound") {
            isCompound = true;

            // Join all elements after the element containing "COMPOUND" into one string element
            // "COMPOUND" can either be at attribute_checker[1] or attribute_checker[2]
            // Attribute checker should then have either 3 elements or 4 elements
            // 3 elements -> [attribute_name, COMPOUND, compound_attributes]
            // 4 elements -> [key_definition, attribute_name, COMPOUND, compound_attributes]
            let index_of_compound = attribute_name
                .iter()
                .position(|&x| x == "compound" || x == "COMPOUND")
                .unwrap(); // Get index of "compound"

            compound_attribute = attribute_name[index_of_compound + 1..].join(" "); // Merge compound attr into one string
            attribute_name.truncate(index_of_compound + 1); // Remove excess elements
            attribute_name.push(&compound_attribute); // Push new compound attribute to end
        }

        let mut data: String = "".to_string();

        match attribute_name.len() {
            1 | 2 => {
                data = statement_data
                    .get(attribute_name[0])
                    .unwrap()
                    .trim()
                    .to_string();
            }
            3 => match isCompound {
                true => {
                    data = statement_data
                        .get(attribute_name[0])
                        .unwrap()
                        .trim()
                        .to_string();
                }
                false => {
                    data = statement_data
                        .get(attribute_name[1])
                        .unwrap()
                        .trim()
                        .to_string();
                }
            }
            4 | 5 => {
                /*
                    * Examples of Attribute
                    * PK/FK userID INTEGER profile(userID)
                    * [PK/AK] full_name COMPOUND (first_name VARCHAR(20), middle_initial CHAR(1), last_name VARCHAR(20))
                    * [FK] MBR COMPOUND (x_min INTEGER, x_max INTEGER, y_min INTEGER, y_max INTEGER) region(coordinates)
                */
                data = statement_data
                    .get(attribute_name[1])
                    .unwrap()
                    .trim()
                    .to_string();
            }
            _ => {}
        }
        let is_compound = true;

        insert_string += &match is_compound {
            true => {
                let compound_attributes = data.split(", ").collect::<Vec<&str>>();
                for (cIndex, cAttribute) in compound_attributes.iter().enumerate() {
                    compound_attribute_string += &if cIndex == compound_attributes.len() - 1 {
                        if cAttribute.parse::<f64>().is_ok() {
                            format!("{})", cAttribute)
                        } else if cAttribute.to_ascii_uppercase() == "NULL" {
                            "NULL)".to_string()
                        } else if cAttribute == &"0" {
                            "0)".to_string()
                        } else if cAttribute.to_ascii_uppercase() == "TRUE" {
                            "TRUE)".to_string()
                        } else if cAttribute.to_ascii_uppercase() == "FALSE" {
                            "FALSE)".to_string()
                        } else {
                            format!("\'{}\')", cAttribute)
                        }
                    } else {
                        if cAttribute.parse::<f64>().is_ok() {
                            format!("{},", cAttribute)
                        } else if cAttribute.to_ascii_uppercase() == "NULL" {
                            "NULL,".to_string()
                        } else if cAttribute == &"0" {
                            "0,".to_string()
                        } else if cAttribute.to_ascii_uppercase() == "TRUE" {
                            "TRUE,".to_string()
                        } else if cAttribute.to_ascii_uppercase() == "FALSE" {
                            "FALSE,".to_string()
                        } else {
                            format!("\'{}\',", cAttribute)
                        }
                    }
                }
                if index == table_attributes.len() - 1 {
                    format!("{});", compound_attribute_string)
                } else {
                    format!("{}, ", compound_attribute_string)
                }
            }
            false => {
                // If we're at the last piece of data, close the statement with a semicolon
                if index == table_attributes.len() - 1 {
                    if data.parse::<f64>().is_ok() {
                        format!("{});", data)
                    } else if data.to_ascii_uppercase() == "NULL" {
                        "NULL);".to_string()
                    } else if data == "0" {
                        "0);".to_string()
                    } else if data.to_ascii_uppercase() == "TRUE" {
                        "TRUE);".to_string()
                    } else if data.to_ascii_uppercase() == "FALSE" {
                        "FALSE);".to_string()
                    } else {
                        format!("\'{}\');", data)
                    }
                    // If we're not at the last piece of data, add a comma to the end of the statement
                } else {
                    if data.parse::<f64>().is_ok() {
                        format!("{}, ", data)
                    } else if data.to_ascii_uppercase() == "NULL" {
                        "NULL, ".to_string()
                    } else if data == "0" {
                        "0, ".to_string()
                    } else if data.to_ascii_uppercase() == "TRUE" {
                        "TRUE, ".to_string()
                    } else if data.to_ascii_uppercase() == "FALSE" {
                        "FALSE, ".to_string()
                    } else {
                        format!("\'{}\', ", data)
                    }
                }
            }
        }
    }
    insert_string
}

pub(crate) fn get_random_data(
    attribute_type: &str,
    optional_data_size: Option<Vec<u16>>,
    statement_data: &HashMap<String, String>,
) -> String {
    /*
        * Generates random data for the attribute type
        * If the attribute type is a custom data type, the custom data type is generated
        * If the attribute type is a default data type, the default data type is generated

        ** Uses the Faker library to generate data for default data types

        :parameters:
            - `attribute_type`: The type of the attribute
            - `optional_data_size`: The optional data size for the attribute
            - `statement_data`: The hashmap of generated data for the table

        :returns:
            - `String`: The generated data for the attribute

        :TODO:
            - Add First Name, Last Name, Middle Name / Middle Initial
    */
    match attribute_type {
        char_type if char_type.starts_with("CHAR") || char_type.starts_with("VARCHAR") => {
            //Attempt to unwrap and get vec[0] of optional data-size, if none, set random value
            let char_size = optional_data_size
                .unwrap_or_else(|| vec![thread_rng().gen_range(3..12)])
                .get(0)
                .unwrap()
                .to_owned();
            Faker
                .fake::<String>()
                .chars()
                .take(char_size as usize)
                .collect::<String>()
        }
        decimal_type if decimal_type.starts_with("DECIMAL") => {
            //Attempt to unwrap and get vec[0] and vec[1] of optional data-size
            //Vec[0] is the number of digits before the decimal point
            //Vec[1] is the number of digits after the decimal point
            //If none, set random values of each
            let unwrapped_decimal = optional_data_size.unwrap_or_else(|| {
                vec![thread_rng().gen_range(3..12), thread_rng().gen_range(3..12)]
            });
            let digits_before_decimal = unwrapped_decimal.get(0).unwrap().to_owned();
            let digits_after_decimal = unwrapped_decimal.get(1).unwrap().to_owned();
            //Create decimal value from digits_before_decimal and digits_after_decimal
            let decimal_value = format!(
                "{}.{}",
                thread_rng().gen_range(0..10_u64.pow(digits_before_decimal as u32)),
                thread_rng().gen_range(0..10_u64.pow(digits_after_decimal as u32)),
            );
            decimal_value
        }
        money_type if money_type.starts_with("MONEY") => {
            //Attempt to unwrap and get vec[0] of optional data-size, if none, set random value
            let dollar_size = optional_data_size
                .unwrap_or_else(|| vec![thread_rng().gen_range(3..12)])
                .get(0)
                .unwrap()
                .to_owned();

            // Generate a dollar amount between 3 figures and either dollar size or 12 figures
            let dollar_amount = thread_rng().gen_range(0..10i32.pow(dollar_size as u32));
            let cents_amount = thread_rng().gen_range(0..100);
            // Create decimal value from dollar_amount and cents_amount
            let decimal_value = format!("{}.{}", dollar_amount, cents_amount);
            decimal_value
        }
        name_type if name_type.starts_with("NAME") => {
            let name = Name(EN).fake::<String>();
            name.replace("'", "")
        }
        password_type if password_type.starts_with("PASSWORD") => Password(std::ops::Range {
            start: 8,
            end: (optional_data_size
                .unwrap_or_else(|| vec![thread_rng().gen_range(8..12)])
                .get(0)
                .unwrap()
                .to_owned() as usize), //Attempt to unwrap and get vec[0] of optional data-size, if none, set random value
        })
            .fake(),
        username_type if username_type.starts_with("USERNAME") => {
            let first_name = FirstName(EN).fake::<String>();
            let last_name = LastName(EN).fake::<String>();
            let mut username = format!("{}{}", first_name, last_name);
            //Check if optional data size is specified in vec[0], if so, truncate username to that size if it is larger
            if let Some(size) = optional_data_size {
                if username.len() > size[0] as usize {
                    username.truncate(size[0] as usize);
                }
            }

            username.replace("'", "")
        }
        "INTEGER" => Faker.fake::<u16>().to_string(),
        "BOOLEAN" => Faker.fake::<bool>().to_string(),
        "DATE" => {
            let year = thread_rng().gen_range(1900..2021);
            let month = thread_rng().gen_range(1..13);
            let day = thread_rng().gen_range(1..29);
            NaiveDate::from_ymd_opt(year, month, day)
                .unwrap()
                .to_string()
        }
        "TIMESTAMP" => {
            let year = thread_rng().gen_range(1900..2021);
            let month = thread_rng().gen_range(1..13);
            let day = thread_rng().gen_range(1..29);
            let hour = thread_rng().gen_range(0..24);
            let minute = thread_rng().gen_range(0..60);
            let second = thread_rng().gen_range(0..60);
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(year, month, day).unwrap(),
                chrono::NaiveTime::from_hms_opt(hour, minute, second).unwrap(),
            )
                .to_string()
        }
        "TIME" => {
            let hour = thread_rng().gen_range(0..24);
            let minute = thread_rng().gen_range(0..60);
            let second = thread_rng().gen_range(0..60);
            chrono::NaiveTime::from_hms_opt(hour, minute, second)
                .unwrap()
                .to_string()
        }
        "GROUP" => ["Member", "Mod"]
            .choose(&mut thread_rng())
            .unwrap()
            .to_string(),
        "EMAIL" => {
            let domains = vec![
                "@outlook.com",
                "@gmail.com",
                "@pitt.edu",
                "@yahoo.com",
                "@proton.mail",
                "@pm.me",
                "@paranoid.email",
            ];
            let username = match (
                statement_data.get(&"name".to_string()),
                statement_data.get(&"full name".to_string()),
                statement_data.get(&"full_name".to_string()),
            ) {
                (Some(name), _, _) => name.replace(" ", ""),
                (_, Some(full_name), _) => full_name.replace(" ", ""),
                (_, _, Some(full_name_underscore)) => full_name_underscore.replace(" ", ""),
                _ => {
                    let name = Name(EN).fake::<String>();
                    name.replace("'", "").to_string();
                    name.replace(" ", "").to_string()
                }
            };
            format!("{}{}", username, domains.choose(&mut thread_rng()).unwrap())
        }
        "STATE_US" => {
            //Generate random state in US using faker
            let state = StateName().fake::<String>();
            state.replace("'", "")
        }
        "CITY_US" => {
            let city = CityName().fake::<String>();
            city.replace("'", "")
        }
        "CITY_SHORT" => {
            let city_prefix = CityPrefix().fake::<String>();
            city_prefix.replace("'", "")
        }
        "STREET_NAME_US" => {
            let street_address = StreetName().fake::<String>();
            street_address.replace("'", "")
        }
        "ZIP_US" => {
            let zip = ZipCode().fake::<String>();
            zip.replace("'", "")
        }
        "SSN" => {
            // Generate random 9 digit number
            let ssn = thread_rng().gen_range(100_000_000..1_000_000_000);
            ssn.to_string()
        }
        "PHONE" => {
            // Generate random phone number using faker
            let phone_number = PhoneNumber().fake::<String>();
            phone_number.replace("'", "")
        }
        "COUNTRY" => {
            // Generate random country using faker
            let country = CountryName().fake::<String>();
            country.replace("'", "")
        }
        "COMPANYNAME" => {
            // Generate random company name using faker
            let company_name = CompanyName().fake::<String>();
            company_name.replace("'", "")
        }
        "INDUSTRY" => {
            // Generate random industry using faker
            let industry = Industry().fake::<String>();
            industry.replace("'", "")
        }
        "PRODUCT" => {
            // Generate random product using faker
            let company_name = CompanyName().fake::<String>();
            let industry = Industry().fake::<String>();
            let word = Word().fake::<String>();
            format!("{} {} {}", company_name, industry, word)
        }
        "PROFESSION" => {
            // Generate random profession using faker
            let profession = Profession().fake::<String>();
            profession.replace("'", "")
        }
        _ => {
            panic!("Unknown Type In Data Generation! {}", attribute_type);
        }
    }
}

pub(crate) fn get_references(attribute_definition: &Vec<String>, index: usize) -> (String, String) {
    /*
        * Gets the referenced table and attribute from the attribute definition
        * Example: PK userID INTEGER profile(userID)
        * Referenced Table: profile
        * Referenced Attribute: userID

        :parameters:
            - `attribute_definition`: The vector of attribute definitions
            - `index`: The index of the attribute definition to get the referenced table and attribute from

        :returns:
            - `String`: The referenced table
            - `String`: The referenced attribute
    */
    let referenced_table = attribute_definition[index]
        .split('(')
        .next()
        .unwrap()
        .to_string();

    let referenced_attribute = attribute_definition[index]
        .split('(')
        .nth(1)
        .unwrap()
        .replace(")", "");

    (referenced_table, referenced_attribute)
}

pub(crate) fn get_referenced_attribute(
    attribute_list: &Vec<String>,
    referenced_attribute: &str,
) -> Option<String> {
    /*
        *Returns the table attribute that is referenced by the foreign key
        *Returns None if the referenced attribute is not found

        :parameters:
            - `attribute_list`: The list of attributes in the table
            - `referenced_attribute`: The referenced attribute to find

        :returns:
            - `Option<String>`: The referenced attribute or None
    */

    if !attribute_list.is_empty() {
        for attribute in attribute_list {
            if attribute.split_whitespace().nth(1) == Some(referenced_attribute) {
                return Some(attribute.to_string());
            }
        }
    }
    None
}



pub fn merge_compound(attribute_definition: &mut Vec<String>) -> bool {
    /*
        * Merges compound attributes into one string
        * Example: 'full_name, COMPOUND, (first_name, VARCHAR(20);, middle_initial CHAR(1):, last_name VARCHAR(20))'
        * Becomes: 'full_name, COMPOUND, (first_name VARCHAR(20); middle_initial CHAR(1); last_name VARCHAR(20))'

        :parameters:
            - `attribute_definition`: The vector of attribute definitions

        :returns:
            - `bool`: True if the attribute definition is a compound attribute, False otherwise
    */
    if attribute_definition.contains(&"compound".to_string())
        || attribute_definition.contains(&"COMPOUND".to_string())
    {
        /*
            * Join All Elements After The Element Containing "COMPOUND" Into One String Element
            * "COMPOUND" Can Either Be At attribute_checker[1] Or attribute_checker[2]
            * Attribute Checker Should Then Have Either 3 Elements Or 4 Elements
            * 3 Elements -> [attribute_name, COMPOUND, compound_attributes]
            * 4 Elements -> [key_definition, attribute_name, COMPOUND, compound_attributes]
            * 5 Elements -> [key_definition, attribute_name, COMPOUND, compound_attributes, foreign_table]
        */
        let mut isReference: bool = false;
        let mut foreign_table: String = String::new();
        let index_of_compound = attribute_definition // Get index of element of string "compound" or "COMPOUND"
            .iter()
            .position(|x| x == "compound" || x == "COMPOUND")
            .unwrap(); // Get index of "compound"

        let index_of_end_compound = if attribute_definition[0].contains("FK") {
            /*
                * Compound Attribute Is A Foreign Key
                * Get Index Of Element Containing "))" (End Of Compound Attribute)
                * This Is Because The Compound Attribute Will Contain The Foreign Table
                * Keeps Foreign Table As Its Own Element
            */
            isReference = true;
            attribute_definition // Get index of element of string ")"
                .iter()
                .position(|x| x.contains("))"))
                .unwrap() // Get index of "))"
        } else {
            /*
                * Not A Foreign Table
                * Proceed As Before
            */
            attribute_definition.len()
        };

        let _compound_attribute = attribute_definition[index_of_compound + 1..index_of_end_compound].join(" "); // Merge compound attr into one string

        /*
            * Remove All Elements After The Element Containing "COMPOUND"
            * Then Push The Compound Attribute To The Attribute Checker
        */
        attribute_definition.truncate(index_of_compound + 1); // Remove excess elements
        attribute_definition.push(_compound_attribute); // Push new compound attribute to end
        if isReference {
            foreign_table = attribute_definition[index_of_end_compound + 1..].join("");
            attribute_definition.push(foreign_table);
        }
        return true;
    }
    false
}

pub(crate) fn set_variable_size(attr_type: &str) -> Option<Vec<u16>> {
    /*
        Improved set_variable_size function
        Still performs the same functionality as the old function
        However this function is able to account for decimal values
        i.e MONEY(10,2) -> 10 is the total number of digits, 2 is the number of digits after the decimal point
        i.e DECIMAL(8,4) -> 8 is the total number of digits, 4 is the number of digits after the decimal point

        :parameters:
            - `attr_type`: The type of the attribute

        :returns:
            - `Option<Vec<u16>>`: The variable size for the attribute returned as a vec
            -  Vec[0] -> Original variable size similar to old function, or number of digits before decimal point
            -  Vec[1] -> Number of digits after decimal point (usually the only reason for index 1 to exist)
            -  None -> No variable size for the attribute
    */
    let some_returned_value: Option<Vec<u16>> = match &attr_type {
        //Check if s.contains("VARIABLE") [in cases of VARIABLE(n)] and s is not "VARIABLE"
        s if s.contains("VARCHAR") && !(*s).eq("VARCHAR") => {
            let variable_size_str = &attr_type[7..];
            let variable_size_str: String =
                variable_size_str[1..variable_size_str.len() - 1].to_string();
            Some(vec![variable_size_str.parse().unwrap(), 0])
        }
        s if s.contains("CHAR") && !(*s).eq("CHAR") => {
            let variable_size_str = &attr_type[4..];
            let variable_size_str = &variable_size_str[1..variable_size_str.len() - 1];
            Some(vec![variable_size_str.parse().unwrap(), 0])
        }
        s if (s.contains("PASSWORD") && !(*s).eq("PASSWORD"))
            || (s.contains("USERNAME") && !(*s).eq("USERNAME")) =>
            {
                let variable_size_str = &attr_type[8..];
                let variable_size_str = &variable_size_str[1..variable_size_str.len() - 1];
                Some(vec![variable_size_str.parse().unwrap(), 0])
            }
        s if s.contains("MONEY") && !(*s).eq("MONEY") => {
            let variable_size_str = &attr_type[5..];
            let variable_size_str = &variable_size_str[1..variable_size_str.len() - 1];
            Some(vec![variable_size_str.parse().unwrap(), 0])
        }
        s if s.contains("DECIMAL") && !(*s).eq("DECIMAL") => {
            let variable_size_str = &attr_type[7..];
            let variable_size_str = &variable_size_str[1..variable_size_str.len() - 1];
            let variable_size_str: Vec<&str> = variable_size_str.split(",").collect();
            Some(vec![
                variable_size_str[0].trim().parse().unwrap(),
                variable_size_str[1].trim().parse().unwrap(),
            ])
        }
        _ => None,
    };
    some_returned_value
}

