#![allow(non_snake_case)]
#![allow(unused_assignments)]

use std::{collections::HashMap, io::{stdout, Write}};

use chrono::{NaiveDate, NaiveDateTime};
use fake::{
    faker::{address::en::*, company::en::*, internet::en::*, name::raw::*, phone_number::en::*},
    locales::*,
    Fake,
    Faker,
    // More modules for mock data found at
    // https://docs.rs/fake/latest/fake/faker/index.html
};

use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::write_to_file;


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

        match isCompound {
            true => {
                let compound_attributes = data.split(", ").collect::<Vec<&str>>();
                for (cIndex, cAttribute) in compound_attributes.iter().enumerate() {
                    if cIndex == compound_attributes.len() - 1 {
                        if cAttribute.parse::<f64>().is_ok() {
                            compound_attribute_string += &format!("{})", cAttribute);
                        } else if cAttribute.to_ascii_uppercase() == "NULL" {
                            compound_attribute_string += "NULL)";
                        } else if cAttribute == &"0" {
                            compound_attribute_string += "0)";
                        } else if cAttribute.to_ascii_uppercase() == "TRUE" {
                            compound_attribute_string += "TRUE)";
                        } else if cAttribute.to_ascii_uppercase() == "FALSE" {
                            compound_attribute_string += "FALSE)";
                        } else {
                            compound_attribute_string += &format!("\'{}\')", cAttribute);
                        }
                    } else {
                        if cAttribute.parse::<f64>().is_ok() {
                            compound_attribute_string += &format!("{},", cAttribute);
                        } else if cAttribute.to_ascii_uppercase() == "NULL" {
                            compound_attribute_string += "NULL,";
                        } else if cAttribute == &"0" {
                            compound_attribute_string += "0,";
                        } else if cAttribute.to_ascii_uppercase() == "TRUE" {
                            compound_attribute_string += "TRUE,";
                        } else if cAttribute.to_ascii_uppercase() == "FALSE" {
                            compound_attribute_string += "FALSE,";
                        } else {
                            compound_attribute_string += &format!("\'{}\',", cAttribute);
                        }
                    }
                }
                if index == table_attributes.len() - 1 {
                    insert_string += &format!("{});", compound_attribute_string);
                } else {
                    insert_string += &format!("{}, ", compound_attribute_string);
                }
            }
            false => {
                // If we're at the last piece of data, close the statement with a semicolon
                if index == table_attributes.len() - 1 {
                    if data.parse::<f64>().is_ok() {
                        insert_string += &format!("{});", data);
                    } else if data.to_ascii_uppercase() == "NULL" {
                        insert_string += "NULL);";
                    } else if data == "0" {
                        insert_string += "0);";
                    } else if data.to_ascii_uppercase() == "TRUE" {
                        insert_string += "TRUE);";
                    } else if data.to_ascii_uppercase() == "FALSE" {
                        insert_string += "FALSE);";
                    } else {
                        insert_string += &format!("\'{}\');", data);
                    }
                    // If we're not at the last piece of data, add a comma to the end of the statement
                } else {
                    if data.parse::<f64>().is_ok() {
                        insert_string += &format!("{}, ", data);
                    } else if data.to_ascii_uppercase() == "NULL" {
                        insert_string += "NULL, ";
                    } else if data == "0" {
                        insert_string += "0, ";
                    } else if data.to_ascii_uppercase() == "TRUE" {
                        insert_string += "TRUE, ";
                    } else if data.to_ascii_uppercase() == "FALSE" {
                        insert_string += "FALSE, ";
                    } else {
                        insert_string += &format!("\'{}\', ", data);
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

pub(crate) fn generate_mock_data(
    tables: &Vec<String>,
    key_dictionary: &HashMap<String, Vec<String>>,
    reference_dictionary: &HashMap<String, Vec<HashMap<String, String>>>,
    path: &String,
    iterations: u16,
) {
    /*
        * Generates the mock data for the tables
        * Writes the mock data to the file specified by path

        :parameters:
            - `tables`: The vector of tables to generate mock data for
            - `key_dictionary`: The hashmap of keys for each table
            - `reference_dictionary`: The hashmap of references for each table
            - `path`: The path to write the mock data to
            - `iterations`: The number of iterations to generate mock data for
    */

    let mut unique_attribute_checker: HashMap<String, Vec<String>> = HashMap::new();
    let mut unique_pair_checker: HashMap<String, Vec<Vec<String>>> = HashMap::new();

    let mut statements_generated: u16 = 0;

    print!(
        "Generating SQL Inserts: {}/{} Created",
        statements_generated, iterations
    );
    stdout().flush().unwrap();

    for table in tables.into_iter() {
        let mut pairwise_table = false;

        let table: Vec<&str> = table.split_whitespace().collect();

        let num_statements = table[0].parse::<i32>().unwrap(); // Keeps Track of Number of Statements to Generate
        let table_name = String::from(table[1]); //Table name
        let mut table_attributes: String = table[2..] //Table attributes
            .join(" ")
            .to_string();

        table_attributes.remove(0); // Remove beginning parenthesis
        table_attributes.remove(table_attributes.len() - 1); // Remove ending parenthesis

        let table_attributes: Vec<String> = table_attributes // Split attributes -> example element: 'PK userID INTEGER'
            .clone()
            .split(",")
            .map(|s| s.to_owned())
            .collect();

        let check_references = reference_dictionary.get(&table_name);
        let attribute_order_keeper: Vec<String> = table_attributes.clone(); // Keeps Track of Order of Attributes. Used for Creating Insert Statements

        let mut primary_keys: Vec<String> = Vec::new();

        // Check if table is a composite keyed table
        if let Some(_check_references) = check_references {
            for attribute in &table_attributes {
                if attribute.trim()[..2].to_uppercase() == "PK".to_string() {
                    primary_keys.push(attribute.to_string());
                }
            }
            if primary_keys.len() > 1 {
                pairwise_table = true;
            }
        }

        for _ in 0..num_statements {
            // Stores generated value for CURRENT Insert statement. Resets after each insert statement is generated
            let mut statement_data: HashMap<String, String> = HashMap::new();

            // Keep track of previous attribute [Used for tables that reference the same table attribute multiple times]
            // (PK/FK userID1 INTEGER profile(userID), PK/FK userID2 INTEGER profile(userID)
            let mut referenced_attributes: HashMap<String, Vec<String>> = HashMap::new();

            // Used in pairwise (composite) key scenarios
            let mut pair_list: Vec<String> = Vec::new();

            for attribute in &table_attributes {
                let pairwise_attribute =
                    if pairwise_table && primary_keys.contains(&attribute.to_string()) {
                        true
                    } else {
                        false
                    };

                let mut attribute_definition: Vec<String> = attribute
                    .trim()
                    .split_whitespace()
                    .map(|s| s.to_owned())
                    .collect();

                let isCompound = merge_compound(&mut attribute_definition);

                let attribute_definition = attribute_definition;

                match &attribute_definition.len() {
                    1 => {
                        /*
                         * User requests default value for attribute
                         * Does not need to specify attribute
                         * For instance if table has attribute that would be NULL for mock data user can put "NULL"
                         * Then for each insert, that attribute will be NULL for
                         */
                        match attribute_definition[0].to_uppercase().as_str() {
                            // TODO - Add more default values
                            "0" | "NULL" | "TRUE" | "FALSE" => {
                                statement_data.insert(
                                    attribute_definition[0].to_string(),
                                    attribute_definition[0].to_string(),
                                );
                            }
                            _ => {
                                println!("[!] Invalid default value for attribute");
                                continue;
                            }
                        }
                    }
                    2 => {
                        /*
                            * Attribute Definition Is Of The Form:
                            * [attribute name] [attribute type]
                            * Example: 'userID INTEGER'

                            * Generate Data For Attribute
                        */

                        let attribute_name = attribute_definition[0].to_string();
                        let attribute_type = attribute_definition[1].to_uppercase().to_string();

                        let optional_variable_size: Option<Vec<u16>> =
                            set_variable_size(&attribute_type);

                        let generated_data: String = get_random_data(
                            &attribute_type,
                            optional_variable_size.clone(),
                            &statement_data,
                        );

                        statement_data.insert(attribute_name, generated_data);
                    }
                    3 => match isCompound {
                        /*
                            * Attribute Definition Is Of The Form:
                            * [key definition] [attribute name] [attribute type]
                            * OR
                            * [attribute name] [compound] [(compound attribute)]
                            * Standard Example: 'PK userID INTEGER'
                            * Compound Example: 'full_name COMPOUND (first_name VARCHAR(20), middle_initial CHAR(1), last_name VARCHAR(20)

                            * Generate Data For Attribute
                        */
                        true => {
                            /*
                             * Compound Attribute. Generate Data For Each Attribute In The Compound Attribute
                             * Example: 'full_name COMPOUND (first_name VARCHAR(20), middle_initial CHAR(1), last_name VARCHAR(20))'
                             */
                            let attribute_name = attribute_definition[0].to_string();
                            // Remove beginning and ending parenthesis of compound attribute
                            let compound_attribute = attribute_definition[2]
                                [1..attribute_definition[2].len() - 1]
                                .to_string();
                            let compound_attribute =
                                compound_attribute.split("; ").collect::<Vec<&str>>();

                            let mut compound_attribute_data: Vec<String> = Vec::new();

                            // Iterate over each attribute in the compound attribute, get type and size, then generate data
                            for attribute in compound_attribute {
                                let attribute =
                                    attribute.split_whitespace().collect::<Vec<&str>>();
                                let attribute_type =
                                    attribute[1].trim().to_uppercase().to_string();

                                let optional_variable_size: Option<Vec<u16>> =
                                    set_variable_size(&attribute_type);

                                let generated_data: String = get_random_data(
                                    &attribute_type,
                                    optional_variable_size.clone(),
                                    &statement_data,
                                );

                                compound_attribute_data.push(generated_data);
                            }

                            // Create string of compound attribute data in form (data1, data2, data3, ...)
                            let compound_attribute_data = compound_attribute_data.join(", ");
                            statement_data.insert(attribute_name, compound_attribute_data);
                        }
                        false => {
                            /*
                             * Standard Attribute. Proceed As Before
                             */
                            let attribute_name = attribute_definition[1].to_string();
                            let attribute_type =
                                attribute_definition[2].to_uppercase().to_string();

                            let optional_variable_size: Option<Vec<u16>> =
                                set_variable_size(&attribute_type);

                            let mut generated_data: String = get_random_data(
                                &attribute_type,
                                optional_variable_size.clone(),
                                &statement_data,
                            );

                            if unique_attribute_checker.contains_key(&attribute_name) {
                                while (unique_attribute_checker[&attribute_name])
                                    .contains(&generated_data)
                                {
                                    generated_data = get_random_data(
                                        &attribute_type,
                                        optional_variable_size.clone(),
                                        &statement_data,
                                    );
                                }
                            }

                            unique_attribute_checker
                                .entry(attribute_name.clone())
                                .or_insert(Vec::new())
                                .push(generated_data.clone());

                            statement_data.insert(attribute_name, generated_data.clone());

                            if pairwise_attribute {
                                pair_list.push(generated_data);
                            }
                        }
                    }
                    4 => match isCompound {
                        /*
                            * Attribute Definition Is Of The Form:
                            * [foreign key definition] [attribute name] [attribute type] [foreign table]
                            * OR
                            * [primary/unique key] [attribute name] [compound] [(compound attribute)]
                            * Example: 'PK/FK userID INTEGER profile(userID)'

                            * Generate Data For Attribute
                        */
                        true => {
                            /*
                                * Keyed Compound Attribute.
                                * Generate Data For Each Attribute In The Compound Attribute
                                * Add to unique_attribute_checker
                            */
                            let comp_attr_name = attribute_definition[1].to_string();
                            let comp_attr_compound = attribute_definition[3].to_string();

                            let comp_attr_compound =
                                comp_attr_compound[1..comp_attr_compound.len() - 1].to_string();

                            let comp_attr_compound =
                                comp_attr_compound.split("; ").collect::<Vec<&str>>();

                            loop {
                                let mut compound_attribute_data: Vec<String> = Vec::new();
                                // Iterate over each attribute in the compound attribute, get type and size, then generate data
                                for attribute in comp_attr_compound.iter() {
                                    let attribute =
                                        attribute.split_whitespace().collect::<Vec<&str>>();
                                    let attribute_type =
                                        attribute[1].trim().to_uppercase().to_string();

                                    let optional_variable_size: Option<Vec<u16>> =
                                        set_variable_size(&attribute_type);

                                    let generated_data: String = get_random_data(
                                        &attribute_type,
                                        optional_variable_size.clone(),
                                        &statement_data,
                                    );

                                    compound_attribute_data.push(generated_data);
                                }

                                // Create string of compound attribute data in form (data1, data2, data3, ...)
                                let compound_attribute_data =
                                    compound_attribute_data.join(", ");

                                // If primary or unique key, add to unique_attribute_checker
                                if unique_attribute_checker.contains_key(&comp_attr_name) &&
                                    unique_attribute_checker[&comp_attr_name].contains(&compound_attribute_data) {
                                    continue;
                                }

                                // Add to unique_attribute_checker
                                unique_attribute_checker
                                    .entry(comp_attr_name.clone())
                                    .or_insert(Vec::new())
                                    .push(compound_attribute_data.clone());

                                // Add to statement_data
                                statement_data.insert(
                                    comp_attr_name.clone(),
                                    compound_attribute_data.clone(),
                                );

                                // If pairwise, add to pair_list
                                if pairwise_attribute {
                                    if pair_list.contains(&compound_attribute_data) {
                                        continue;
                                    }
                                    pair_list.push(compound_attribute_data.clone());
                                }
                                break;
                            }
                        }
                        false => {
                            /*
                                * Standard Foreign Key Attribute.
                                * Generate Data For Attribute Based On Referenced Attribute
                                * I.E. Use Previously Generated Data From Referenced Attribute
                            */
                            let (referenced_table, referenced_attribute) =
                                get_references(&attribute_definition, 3);

                            if get_referenced_attribute(
                                key_dictionary.get(&referenced_table).unwrap(),
                                &referenced_attribute.to_uppercase().to_string(),
                            )
                                .is_none()
                            {
                                println!(
                                    "\nPROGRAM ERROR IN GENERATING DATA [Getting Referenced Attribute]"
                                );
                                std::process::exit(1);
                            }

                            loop {
                                /*
                                    * Reference does exist and is valid.
                                    * Get a random reference for that attribute from the list
                                 */
                                let randomized_data = unique_attribute_checker
                                    .get(&referenced_attribute.to_string())
                                    .unwrap()
                                    .choose(&mut thread_rng())
                                    .unwrap()
                                    .to_string();

                                /*
                                    * Check if the randomized data is the same as the previous data
                                    * If it is, generate new data
                                    * [key definition][0] [attribute name][1] [attribute type][2] [foreign table][3]
                                 */
                                if referenced_attributes
                                    .contains_key(&referenced_attribute.to_string())
                                {
                                    if referenced_attributes
                                        .get(&attribute_definition[3].to_string())
                                        .unwrap()
                                        .contains(&attribute_definition[1].to_string())
                                    {
                                        println!("Data being generated for same attribute.");
                                        println!("Program error.");
                                        println!("Exiting program.");

                                        std::process::exit(1);
                                    } else {
                                        if referenced_attributes
                                            .get(&attribute_definition[3].to_string())
                                            .unwrap()
                                            .contains(&randomized_data)
                                        {
                                            continue;
                                        }
                                    }
                                }

                                /*
                                    * True if table is composite keyed
                                    * Pair list can't have same data for keyed attributes if it does, regenerate data for current attribute
                                */
                                if pairwise_attribute {
                                    if pair_list.contains(&randomized_data) {
                                        continue;
                                    }
                                    pair_list.push(randomized_data.clone());
                                } else {
                                    //Not pairwise, check if data is unique for attribute or continue loop and generate new data for attribute
                                    if attribute_definition[0].starts_with("PK")
                                        || attribute_definition[0].starts_with("AK")
                                    {
                                        if unique_attribute_checker
                                            .contains_key(&attribute_definition[1])
                                            && unique_attribute_checker
                                            [&attribute_definition[1]]
                                            .contains(&randomized_data)
                                        {
                                            continue;
                                        }

                                        unique_attribute_checker
                                            .entry(attribute_definition[1].to_string())
                                            .or_insert(Vec::new())
                                            .push(randomized_data.clone());
                                    }
                                }

                                /*
                                    * Add the attribute to the list of referenced attributes
                                    * Add the data to the statement data
                                    * Break out of the loop
                                 */
                                referenced_attributes
                                    .entry(attribute_definition[3].to_string())
                                    .or_insert(Vec::new())
                                    .push(attribute_definition[1].to_string());

                                statement_data.insert(
                                    attribute_definition[1].to_string(),
                                    randomized_data,
                                );
                                break;
                            }
                        }
                    }
                    5 => {
                        /*
                            * This only runs when its a compound attribute with a foreign key
                            * Attribute Definition Is Of The Form:
                            * [key definition][0] [attribute name][1] [compound][2] [(compound attribute)][3] [foreign table][4]
                        */
                        let (referenced_table, referenced_attribute) =
                            get_references(&attribute_definition, 4);

                        if get_referenced_attribute(
                            key_dictionary.get(&referenced_table).unwrap(),
                            &referenced_attribute.to_uppercase().to_string(),
                        )
                            .is_none()
                        {
                            println!(
                                "\nPROGRAM ERROR IN GENERATING DATA [Getting Referenced Attribute]"
                            );
                            std::process::exit(1);
                        }

                        loop {
                            /*
                                * Reference does exist and is valid.
                                * Get a random reference for that attribute from the list
                            */
                            let randomized_data = unique_attribute_checker
                                .get(&referenced_attribute.to_string())
                                .unwrap()
                                .choose(&mut thread_rng())
                                .unwrap()
                                .to_string();

                            if referenced_attributes
                                .contains_key(&referenced_attribute.to_string())
                            {
                                if referenced_attributes
                                    .get(&attribute_definition[4].to_string())
                                    .unwrap()
                                    .contains(&attribute_definition[1].to_string())
                                {
                                    println!("Data being generated for same attribute.");
                                    println!("Program error.");
                                    println!("Exiting program.");

                                    std::process::exit(1);
                                } else {
                                    if referenced_attributes
                                        .get(&attribute_definition[4].to_string())
                                        .unwrap()
                                        .contains(&randomized_data)
                                    {
                                        continue;
                                    }
                                }
                            }

                            /*
                               * True if table is composite keyed
                               * Pair list can't have same data for keyed attributes if it does, regenerate data for current attribute
                            */
                            if pairwise_attribute {
                                if pair_list.contains(&randomized_data) {
                                    continue;
                                }
                                pair_list.push(randomized_data.clone());
                            } else {
                                //Not pairwise, check if data is unique for attribute or continue loop and generate new data for attribute
                                if attribute_definition[0].starts_with("PK")
                                    || attribute_definition[0].starts_with("AK")
                                {
                                    if unique_attribute_checker
                                        .contains_key(&attribute_definition[1])
                                        && unique_attribute_checker[&attribute_definition[1]]
                                        .contains(&randomized_data)
                                    {
                                        continue;
                                    }

                                    unique_attribute_checker
                                        .entry(attribute_definition[1].to_string())
                                        .or_insert(Vec::new())
                                        .push(randomized_data.clone());
                                }
                            }

                            /*
                                * Add the attribute to the list of referenced attributes
                                * Add the data to the statement data
                                * Break out of the loop
                             */
                            referenced_attributes
                                .entry(attribute_definition[4].to_string())
                                .or_insert(Vec::new())
                                .push(attribute_definition[1].to_string());

                            statement_data.insert(
                                attribute_definition[1].to_string(),
                                randomized_data,
                            );
                            break;

                        }
                    }
                    _ => {
                        println!("\nPROGRAM ERROR IN GENERATING DATA [Attribute Definition Length]");
                        std::process::exit(1);
                    }
                }
            }
            /*
                * Check if the table is a pairwise table
                * If it is, check if the pair has been generated before
                * If it has, generate new data for the pair
             */

            if pairwise_table {
                //If pairwise table, make sure composite key is not already existent
                if !unique_pair_checker.contains_key(&table_name) {
                    //True if table does not exist in unique_pair_checker
                    unique_pair_checker
                        .entry(table_name.clone())
                        .or_insert(Vec::new())
                        .push(pair_list.clone());
                } else {
                    // Composite table exists, check if pair has been generated before
                    let (pair_changed, new_pair) = check_pair(
                        &pair_list,
                        &unique_pair_checker[&table_name],
                        &attribute_order_keeper,
                        &unique_attribute_checker,
                        0,
                    );
                    if pair_changed {
                        // Pair did exist and new data was generated in check_pair, change data in statement_data to match
                        // Rewrite the data for the composite key attributes in the statement data
                        let mut index = 0; // Used to match the new data with the correct attribute
                        for attribute in attribute_order_keeper.iter() {
                            let attribute_definitions: Vec<&str> =
                                attribute.split_ascii_whitespace().collect();

                            if primary_keys.contains(&attribute.to_string()) {
                                // Attribute is part of composite key
                                // Rewrite the data for the attribute
                                statement_data.insert(
                                    attribute_definitions[1].to_string(),
                                    new_pair[index].to_string(),
                                );
                                index += 1;
                            }
                        }
                    }

                    // Add the new pair to the unique_pair_checker
                    unique_pair_checker
                        .entry(table_name.clone())
                        .or_insert(Vec::new())
                        .push(new_pair.clone());
                }
            }

            // Write the insert statement to the file. Executes each time, will modify later for improved time complexity
            match write_to_file(
                create_insert_statement(&table_name, &table_attributes, &statement_data),
                path,
            ) {
                Ok(_) => {
                    // Successfully wrote to file, increment statements_generated and print progress
                    statements_generated += 1;
                    print!(
                        "\rGenerating SQL Inserts: {}/{} Created",
                        statements_generated, iterations
                    );
                    stdout().flush().unwrap();
                }
                Err(_) => {
                    // Usually implies bigger error, just exit the program
                    println!("[!] Unable to write to file");
                    std::process::exit(1);
                }
            }
        }
    }
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
