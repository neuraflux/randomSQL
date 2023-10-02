#![allow(non_snake_case)]
#![allow(unused_assignments)]

use chrono::{NaiveDate, NaiveDateTime};
use fake::{
    faker::{internet::en::*, name::raw::*, address::en::*, phone_number::en::*, company::en::*},
    locales::*,
    Fake, Faker,
    // More modules for mock data found at
    // https://docs.rs/fake/latest/fake/faker/index.html
};

use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io,
    io::{stdout, Write},
};

fn check_data_type(attribute_type: &str) -> bool {
    /*
       * Checks if the data type is valid
       * Returns true if valid, false if invalid
       * Custom data types made for this program: EMAIL, GROUP, PHONE, SSN, STATE, ZIP, STREET-ADDRESS, FULL-ADDRESS, NAME, PASSWORD, USERNAME

       :Parameters:
           - `attribute_type`: The data type to check
       :Returns:
           - `bool`: True if valid, false if invalid

    */

    let valid_types = [
        "BIGINT",
        "BIT",
        "BOOLEAN",
        "BOX",
        "BYTEA",
        "CHAR",
        "CIDR",
        "CIRCLE",
        "CITY_US",
        "CITY_SHORT",
        "COUNTRY",
        "COMPANYNAME",
        "EMAIL",
        "DATE",
        "DOUBLE PRECISION",
        "ENUM",
        "FLOAT4",
        "FLOAT8",
        "GROUP",
        "INDUSTRY",
        "INET",
        "INTEGER",
        "NAME",
        "INTERVAL",
        "JSON",
        "JSONB",
        "LINE",
        "LSEG",
        "MACADDR",
        "MONEY",
        "NUMERIC",
        "PASSWORD",
        "PATH",
        "PG_LSN",
        "PHONE",
        "POINT",
        "POLYGON",
        "PROFESSION",
        "REAL",
        "SMALLINT",
        "SERIAL",
        "SSN",
        "STATE",
        "STATE_US",
        "STREET_NAME_US",
        "STREET_ADDRESS",
        "TEXT",
        "TIME",
        "TIMESTAMP",
        "TSQUERY",
        "TSVECTOR",
        "TXID_SNAPSHOT",
        "USERNAME",
        "UUID",
        "XML",
        "VARCHAR",
        "ZIP_US",
    ];

    // Make sure the data type is valid
    valid_types
        .iter()
        .any(|&subtring| attribute_type.starts_with(subtring))
}

fn check_key_definition(key_def: &str) -> bool {
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

fn check_pair(
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

    if previous_pairs.contains(&generated_pair_vector) || previous_pairs.contains(&reverse_generated_pair) {
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

fn create_insert_statement(
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
    */

    let mut insert_string = format!("INSERT INTO {} VALUES (", table_name);

    // Iterate over the generated data and add it to the INSERT statement
    // useless_key has no purpose and is only there to aid in iteration.
    for (index, attribute) in table_attributes.iter().enumerate() {
        let attribute_name: Vec<&str> = attribute.trim().split(" ").collect();
        let mut data: String = "".to_string();

        match attribute_name.len() {
            1 | 2=> {
                data = statement_data.get(attribute_name[0]).unwrap().trim().to_string();
            }
            3 | 4 => {
                data = statement_data.get(attribute_name[1]).unwrap().trim().to_string();
            }
            _ => {}
        }

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
    insert_string
}

fn get_random_data(
    attribute_type: &str,
    optional_data_size: Option<u16>,
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
    */
    match attribute_type {
        char_type if char_type.starts_with("CHAR") || char_type.starts_with("VARCHAR") => {
            let char_size = optional_data_size.unwrap_or_else(|| thread_rng().gen_range(8..30));
            Faker
                .fake::<String>()
                .chars()
                .take(char_size as usize)
                .collect::<String>()
        }
        money_type if money_type.starts_with("MONEY") => {
            let dollar_size = optional_data_size.unwrap_or_else(|| thread_rng().gen_range(3..12));
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
            end: (optional_data_size.unwrap_or(30) as usize),
        })
        .fake(),
        username_type if username_type.starts_with("USERNAME") => {
            let first_name = FirstName(EN).fake::<String>();
            let last_name = LastName(EN).fake::<String>();
            let mut username = format!("{}{}", first_name, last_name);
            //Check if optional data size is specified, if so, truncate username to that size if it is larger
            if let Some(data_size) = optional_data_size {
                if username.len() > data_size as usize {
                    username =  username[..data_size as usize].to_string();
                }
            }

            username.replace("'", "")
        }
        "INTEGER" => Faker.fake::<u16>().to_string(),
        "FLOAT" => Faker.fake::<f32>().to_string(),
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

fn get_referenced_attribute(
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

fn generate_mock_data(
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
        table_attributes.remove(0); // Remove beginning paranthesis
        table_attributes.remove(table_attributes.len() - 1); // Remove ending paranthesis
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

        for _ in 0..num_statements.clone() {
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

                let attribute_definition: Vec<&str> = attribute.trim().split(' ').collect();

                match &attribute_definition.len() {
                    1 => {
                        /*
                         * User requests default value for attribute
                         * Does not need to specify attribute
                         * For instance if table has attribute that would be NULL for mock data user can put "NULL"
                         * Then for each insert, that attribute will be NULL for
                         */
                        match attribute_definition[0].to_uppercase().as_str() {
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

                        let optional_variable_size: Option<u16> =
                            set_variable_size(&attribute_type);

                        let generated_data: String = get_random_data(
                            &attribute_type,
                            optional_variable_size,
                            &statement_data,
                        );

                        statement_data.insert(attribute_name, generated_data);
                    }
                    3 => {
                        /*
                            * Attribute Definition Is Of The Form:
                            * [key definition] [attribute name] [attribute type]
                            * Example: 'PK userID INTEGER'

                            * Generate Data For Attribute
                        */

                        let attribute_name = attribute_definition[1].to_string();
                        let attribute_type = attribute_definition[2].to_uppercase().to_string();

                        let optional_variable_size: Option<u16> =
                            set_variable_size(&attribute_type);

                        let mut generated_data: String = get_random_data(
                            &attribute_type,
                            optional_variable_size,
                            &statement_data,
                        );

                        if unique_attribute_checker.contains_key(&attribute_name) {
                            while (unique_attribute_checker[&attribute_name])
                                .contains(&generated_data)
                            {
                                generated_data = get_random_data(
                                    &attribute_type,
                                    optional_variable_size,
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
                    4 => {
                        /*
                            * Attribute Definition Is Of The Form:
                            * [key definition] [attribute name] [attribute type] [foreign table]
                            * Example: 'PK/FK userID INTEGER profile(userID)'

                            * Generate Data For Attribute
                        */
                        let referenced_table = &attribute_definition[3]
                            .split('(')
                            .next()
                            .unwrap()
                            .to_string();

                        let referenced_attribute = &attribute_definition[3]
                            .split('(')
                            .nth(1)
                            .unwrap()
                            .replace(")", "");

                        if get_referenced_attribute(
                            key_dictionary.get(referenced_table).unwrap(),
                            &referenced_attribute.to_uppercase().to_string(),
                        )
                        .is_none()
                        {
                            println!("\nPROGRAM ERROR IN GENERATING DATA [Getting Referenced Attribute]");
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
                             */

                            if referenced_attributes.contains_key(&referenced_attribute.to_string())
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
                            True if table is composite keyed
                            Pair list can't have same data for keyed attributes if it does, regenerate data for current attribute
                            */
                            if pairwise_attribute {
                                if pair_list.contains(&randomized_data) {
                                    continue;
                                }
                                pair_list.push(randomized_data.clone());
                            } else { //Not pairwise, check if data is unique for attribute or continue loop and generate new data for attribute
                                if attribute_definition[0].starts_with("PK")
                                    || attribute_definition[0].starts_with("AK")
                                {
                                    if unique_attribute_checker
                                        .contains_key(attribute_definition[1].clone())
                                        && unique_attribute_checker[attribute_definition[1].clone()]
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

                            statement_data
                                .insert(attribute_definition[1].to_string(), randomized_data);
                            break;
                        }
                    }
                    _ => {
                        println!("\nPROGRAM ERROR IN GENERATING DATA [Attribute Defintion Length]");
                        std::process::exit(1);
                    }
                }
            }
            /*
             * Check if the table is a pairwise table
             * If it is, check if the pair has been generated before
             * If it has, generate new data for the pair
             */

            if pairwise_table { //If pairwise table, make sure composite key is not already existent
                if !unique_pair_checker.contains_key(&table_name) { //True if table does not exist in unique_pair_checker
                    unique_pair_checker
                        .entry(table_name.clone())
                        .or_insert(Vec::new())
                        .push(pair_list.clone());
                } else { // Composite table exists, check if pair has been generated before
                    let (pair_changed, new_pair) = check_pair(
                        &pair_list,
                        &unique_pair_checker[&table_name],
                        &attribute_order_keeper,
                        &unique_attribute_checker,
                        0,
                    );
                    if pair_changed { // Pair did exist and new data was generated in check_pair, change data in statement_data to match
                        // Rewrite the data for the composite key attributes in the statement data
                        let mut index = 0; // Used to match the new data with the correct attribute
                        for attribute in attribute_order_keeper.iter() {
                            let attribute_definitions: Vec<&str> = attribute.split_ascii_whitespace().collect();

                            if primary_keys.contains(&attribute.to_string()) { // Attribute is part of composite key
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

fn set_variable_size(attr_type: &str) -> Option<u16> {
    /*
    Returns Either variable_size or None

    :param attr_type is the type being parsed
    :return some_returned_value: Option<u16>
    */
    let some_returned_value: Option<u16> = match &attr_type {
        s if s.contains("VARCHAR") => {
            let variable_size_str = &attr_type[7..];
            let variable_size_str: String =
                variable_size_str[1..variable_size_str.len() - 1].to_string();
            Some(variable_size_str.parse().unwrap())
        }
        s if s.contains("CHAR") => {
            let variable_size_str = &attr_type[4..];
            let variable_size_str = &variable_size_str[1..variable_size_str.len() - 1];
            Some(variable_size_str.parse().unwrap())
        }
        s if s.contains("PASSWORD") || s.contains("USERNAME") => {
            let variable_size_str = &attr_type[8..];
            let variable_size_str = &variable_size_str[1..variable_size_str.len() - 1];
            Some(variable_size_str.parse().unwrap())
        }
        s if s.contains("MONEY") => {
            let variable_size_str = &attr_type[5..];
            let variable_size_str = &variable_size_str[1..variable_size_str.len() - 1];
            Some(variable_size_str.parse().unwrap())
        }
        _ => None,
    };
    some_returned_value
}

fn write_to_file(generated_statement: String, path: &String) -> io::Result<()> {
    /*
        * Writes the generated statement to the file specified by path variable
        * If the file does not exist, it is created
        * If the file does exist, the generated statement is appended to the file

        :parameters:
            - `generated_statement`: The generated statement to write to the file
            - `path`: The path to write the generated statement to

        :returns:
            - `io::Result<()>`: The result of writing to the file
    */

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&path)
        .unwrap();

    writeln!(file, "{}", generated_statement)?;

    Ok(())
}

fn main() {
    /*
     * Main function for the program
     * Takes command to input sql tables with their attributes to create mock data
     * Able to produce mock data for multiple tables at once up
     * Capable of Reference Integrity and Unique/Keyed Attributes
     */

    let mut custom_path: Option<String> = None;
    let mut tuples: Vec<String> = Vec::new();
    let mut key_dictionary: HashMap<String, Vec<String>> = HashMap::new();
    let mut reference_dictionary: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();
    let mut total_iterations: u16 = 0;

    display_help(false);

    loop {
        print!("[*] Manage Tables --> ");
        let _ = std::io::stdout().flush();
        let mut sql_input: String = String::new();
        std::io::stdin()
            .read_line(&mut sql_input)
            .expect("Failed to read SQL command");

        match sql_input.trim().to_lowercase().as_str() {
            // There's at least one table to generate mock data for
            "generate" if !tuples.is_empty() => {
                if custom_path.is_none() {
                    let user_folder = dirs::home_dir().unwrap();
                    //Set default path to users documents directory and create/overwrite file named sample-data.sql
                    //Use shlex so it works for any file system i.e Mac and Linux -> /Users/username/Documents/sample-data.sql Windows -> C:\Users\username\Documents\sample-data.sql
                    let documents_dir = user_folder.join("Documents");
                    let file_path = documents_dir.join("sample-data.sql");
                    custom_path = Some(file_path.to_str().unwrap().to_string());
                }

                let custom_path = custom_path.unwrap();
                fs::write(&custom_path, "").expect("Unable to write to file");

                println!("[*] Generating Mock Data...");
                generate_mock_data(
                    &tuples,
                    &key_dictionary,
                    &reference_dictionary,
                    &custom_path,
                    total_iterations,
                );

                //Tell user where the file is located
                println!("\n[*] Mock Data Generated In '{}'", custom_path);
                break;
            }
            // No tables to generate mock data for
            "generate" => {
                println!("[!] No Tables to Generate Mock Data For");
                continue;
            }
            // Clear the terminal
            "clear" => {
                print!("{}[2J", 27 as char);
                continue;
            }
            // Display entire help menu
            "help" => {
                display_help(true);
                continue;
            }
            // Exit the program
            "exit" | "quit" => {
                println!(
                    "All tables and mock data will be lost. Are you sure you want to exit? (y/n)"
                );
                let _ = std::io::stdout().flush();
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read SQL command");
                if input.trim().to_lowercase().as_str() == "y" {
                    println!("[*] Exiting...");
                    break;
                } else {
                    continue;
                }
            }
            _ => {
                sql_input = sql_input.replace("\\", "\\\\");
                let command_tokens = shlex::split(&sql_input.trim()).unwrap_or_default();
                let sql_command_list: Vec<&str> =
                    command_tokens.iter().map(|x| x.as_str()).collect();
                if sql_command_list.len() < 2 {
                    println!("[!] Invalid SQL Command");
                    continue;
                }

                match sql_command_list[0].to_lowercase().as_str() {
                    "path" => {
                        //get path from sql_command_list[1] then check if directory and if the file already exists. If it does, ask to overwrite. If not, create the file
                        let path: &str = sql_command_list[1].trim();

                        if std::path::Path::new(&path).exists() {
                            println!("[!] File already exists. Overwrite? (y/n)");
                            let _ = std::io::stdout().flush();
                            let mut input = String::new();
                            std::io::stdin()
                                .read_line(&mut input)
                                .expect("Failed to read SQL command");
                            if input.trim().to_lowercase().as_str() == "y" {
                                fs::write(&path, "").expect("Unable to write to file");
                                println!("[*] File overwritten");
                            } else {
                                println!("[*] File not overwritten");
                                continue;
                            }
                        } else {
                            fs::write(&path, "").expect("Unable to write to file");
                            println!("[*] File created");
                        }
                        custom_path = Some(path.to_string());
                    }
                    "add" => {
                        let mut valid: bool = true;
                        let value = sql_command_list[1].parse::<u16>();
                        match value {
                            Ok(iterations) => {
                                if iterations <= 0 {
                                    println!("[!] Invalid Number of Iterations");
                                    valid = false;
                                    break;
                                }
                                total_iterations += iterations;
                            }
                            Err(_) => {
                                println!("[!] Error With Iteration Input");
                                valid = false;
                                break;
                            }
                        };

                        let attributes = sql_command_list[3..sql_command_list.len()].join(" ");
                        let attributes: String = attributes[1..attributes.len() - 1].to_string();
                        let attributes = attributes.split(",").collect::<Vec<&str>>();

                        // Go through each elemeent in the attributes vector and trim it before the for loop below
                        let attributes = attributes
                            .iter()
                            .map(|elem| elem.trim())
                            .collect::<Vec<&str>>();

                        println!("Attributes: {:?}", attributes);

                        for attribute in attributes {
                            let attribute_checker =
                                attribute.trim().split(" ").collect::<Vec<&str>>();
                            match attribute_checker.len() {
                                1 => match attribute_checker[0].to_uppercase().as_str() {
                                    "0" | "NULL" | "TRUE" | "FALSE" => {
                                        println!("[!] Valid default value for attribute");
                                    }
                                    _ => {
                                        println!("[!] Invalid default value for attribute");
                                        valid = false;
                                        break;
                                    }
                                },
                                2 => {
                                    if !check_data_type(
                                        attribute_checker[1].to_uppercase().as_str(),
                                    ) {
                                        println!("[!] Invalid Data Type");
                                        valid = false;
                                        break;
                                    }
                                }
                                3 => {
                                    if !check_key_definition(attribute_checker[0].to_uppercase().as_str()) {
                                        println!("[!] Invalid Key Definition");
                                        valid = false;
                                        break;
                                    }
                                    if !check_data_type(
                                        attribute_checker[2].to_uppercase().as_str(),
                                    ) {
                                        println!("[!] Invalid Data Type");
                                        valid = false;
                                        break;
                                    }
                                    if attribute_checker[0].to_uppercase() == "PK" || attribute_checker[0].to_uppercase() == "AK"
                                    {
                                        key_dictionary
                                            .entry(sql_command_list[2].to_string())
                                            .or_insert(Vec::new())
                                            .push(attribute.to_uppercase().to_string());
                                    }
                                }
                                4 => {
                                    if !check_key_definition(attribute_checker[0].to_uppercase().as_str()) {
                                        println!("[!] Invalid Key Definition");
                                        valid = false;
                                        break;
                                    }
                                    if !check_data_type(
                                        attribute_checker[2].to_uppercase().as_str(),
                                    ) {
                                        println!("[!] Invalid Data Type");
                                        valid = false;
                                        break;
                                    }

                                    let referencer =
                                        attribute_checker[3].split("(").collect::<Vec<&str>>();
                                    let table_reference = referencer[0].trim();
                                    let attribute_name = referencer[1].trim().replace(")", "");

                                    let referenced_attribute = get_referenced_attribute(
                                        &key_dictionary
                                            .get(&table_reference.to_string())
                                            .unwrap_or(&vec![]),
                                        &attribute_name.to_uppercase().to_string(),
                                    );

                                    match referenced_attribute {
                                        Some(_) => {
                                            if attribute_checker[0].to_uppercase() == "PK"
                                                || attribute_checker[0].to_uppercase() == "AK"
                                            {
                                                key_dictionary
                                                    .entry(sql_command_list[2].to_string())
                                                    .or_insert(Vec::new())
                                                    .push(attribute.to_string());
                                            }
                                            reference_dictionary
                                                .entry(sql_command_list[2].to_string())
                                                .or_insert(Vec::new())
                                                .push(HashMap::from([(
                                                    table_reference.to_string(),
                                                    attribute_name.to_string(),
                                                )]));
                                        }
                                        None => {
                                            println!("[!] Invalid Reference");
                                            valid = false;
                                            break;
                                        }
                                    }
                                }
                                _ => {
                                    println!("Error In Attributes For The Table: {}...Attribute: {} Is Not Formatted Properly", sql_command_list[2], attribute);
                                    valid = false;
                                    break;
                                }
                            }
                        }
                        if valid {
                            let table_string = sql_command_list[1..sql_command_list.len()].join(" ");
                            tuples.push(table_string);
                            println!(
                                "[*] {} Insert Statements Added For {}",
                                sql_command_list[1], sql_command_list[2]
                            );
                        } else {
                            continue;
                        }
                    }
                    "remove" | "rm" => {
                        let table_to_delete = tuples
                            .iter()
                            .find(|elem| {
                                elem.split_whitespace().nth(1) == Some(sql_command_list[1])
                            })
                            .cloned();
                        if let Some(table) = table_to_delete {
                            let num_statements = table.split_whitespace().nth(0).unwrap();
                            total_iterations -= num_statements.parse::<u16>().unwrap();
                            tuples.retain(|elem| elem != &table);
                        } else {
                            println!("[!] Table Not Found");
                        }
                    }
                    "modify" | "mod" => {
                        let table_to_modify = tuples
                            .iter()
                            .find(|elem| {
                                elem.split_whitespace().nth(1) == Some(sql_command_list[1])
                            })
                            .cloned();
                        // Find attribute name at sql_command_list[2] and replace the entire attribute with sql_command_list[3..sql_command_list.len()]
                        // TODO

                    }
                    "show" => {
                        // If len == 3 Then A Specific Table Has Been Given To Pull Data From
                        // Else
                        let specifier = if sql_command_list.len() == 3 {
                            Some(sql_command_list[2].to_lowercase())
                        } else {
                            None
                        };

                        match sql_command_list[1].to_lowercase().as_str() {
                            // Show Every Table The Program Will Generate Inserts For [Unless Given A Specific Table]
                            "inserts" => {
                                for table in &tuples {
                                    if specifier.is_none()
                                        || *specifier.as_ref().unwrap()
                                            == table.split(' ').nth(1).unwrap().to_lowercase()
                                    {
                                        println!(
                                            "Table -> {}: Associated Tuple -> {}",
                                            table.split(' ').nth(1).unwrap(),
                                            table
                                        );
                                    }
                                }
                            }
                            // Show The Keys For Every Table [Unless Given A Specific Table]
                            "keys" => {
                                for (table, keys) in &key_dictionary {
                                    if specifier.is_none()
                                        || *specifier.as_ref().unwrap()
                                            == table.split(' ').nth(1).unwrap().to_lowercase()
                                    {
                                        println!("Keys For Table {} -> {:?}", table, keys);
                                    }
                                }
                            }
                            // Show's All Tables That Reference Another Table [Unless Given A Specific Table]
                            "references" | "refs" => {
                                for (ref_table, key) in &reference_dictionary {
                                    if specifier.is_none()
                                        || *specifier.as_ref().unwrap()
                                            == ref_table.split(' ').nth(1).unwrap().to_lowercase()
                                    {
                                        for value in key {
                                            for (r_key, r_value) in value {
                                                println!("Referencing Table -> {} References {} From Table {}", ref_table, r_value, r_key);
                                            }
                                        }
                                    }
                                }
                            }
                            // Show's All Custom Types Made Specifically For This Program
                            "types" => {
                                println!(
                                    "
                                        Custom Types Created For This Program
                                        NAME -> Name is a custom type used to replace typical string DataTypes in sql for
                                        when the user wishes to generate a realistic name, assigning a typical
                                        VARCHAR(length) to an attribute where you wish to be a name may result in a randomly
                                        generated string that does not represent a persons full name in real life

                                        EMAIL -> Email is another custom type used to replace string DataTypes in sql. This
                                        ensures that every value generated for said attribute will follow standard email
                                        format 'abc@abc.abc' and also excludes symbols that are commonly excluded in
                                        deployed email domain services (^&% etc) 

                                        PASSWORD(N) -> Password is another custom type used to replace string DataTypes in sql.
                                        While it generates similar strings that would be generated should you just do a
                                        VARCHAR(length) variable in its place, PASSWORD types have the added step of 
                                        ensuring common password complexity constraints are enforced such as
                                            Password Must Include At Least: 
                                                [1 Capital Letter, 1 Lowercase, 1 Number, 1 Special Char]
                                                [N = Max Length Of Password And It Generates Length Between 8..N]
                                                [NOTE] IF YOU DO NOT ADD A LENGTH TO PASSWORD I.E PASSWORD(40)
                                                [THEN] THE PROGRAM WILL PANIC. THIS INCLUDES STANDARD VARCHAR(N) AND CHAR(N) TYPES

                                        GROUP -> Is a custom DataType that technically would replace String SQL types.
                                        Assigning this type to an attribute will assign either 'mod' or 'member' as a role

                                        USERNAME(N) -> Is a custom DataType that technically would replace String SQL types.
                                        Assigning this type to an attribute will assign a randomly generated username

                                        MONEY(N) -> Is a shorthand for designating decimal values of xxx.xx
                                        All values will only have 2 decimal places between 0..99
                                        N -> Max figure for the money value (i.e MONEY(7) generates values between 0.00 and 9,999,999.99)

                                        ## NUMEROUS OTHER TYPES HAVE BEEN ADDED THAT I HAVE YET TO ADD DESCRIPTIONS FOR ##
                                    "
                                        );
                            }
                            // Show's Examples Of Commands To Aid The User Optional [Specifier] Shows Only Specific Examples
                            // If Specifier Not Given. All Examples Are Shown
                            "examples" => {
                                // Show's Examples Of How To Add Tables
                                if specifier.is_none() || specifier == Some("add".to_owned()) {
                                    // Examples For Add
                                    println!("
                                        Add Example [Adding 'profile' Table To List]:
                                        [Add Example #1] -> add 100 profile (PK userID INTEGER, name NAME, AK email EMAIL, password PASSWORD(30), dateOfBirth DATE, lastLogin TIMESTAMP)
                                    ");
                                }
                                // Show's Examples Of How To Delete Tables
                                if specifier.is_none() || specifier == Some("del".to_owned()) {
                                    // Examples For Del
                                    println!(
                                        "
                                        Rm Example [Assuming 'profile' Table Is Defined]:
                                        [Rm Example #1] -> Rm profile
                                    "
                                    );
                                }
                                // Show's Examples Of How To Modify Tables
                                if specifier.is_none()
                                    || specifier == Some("modify".to_owned())
                                    || specifier == Some("mod".to_owned())
                                {
                                    // Examples For Modify
                                    println!(
                                        "
                                        Modify Example [Assuming 'profile' Table Is Defined From 'Add' Example]:
                                        [In This Case This Example Converts The PK 'email' Attribute To A AK]
                                        
                                        [Current Attribute Definition] -> PK email EMAIL
                                        [Mod Example #1] -> mod profile email AK email EMAIL
                                        [New Attribute Definition] -> AK email EMAIL

                                        [NOTE] The Attribute Definition Must Be In The Same Format As The Original
                                        "
                                    );
                                }
                                // Explains Reference Formatting To Aid In Table Input
                                if specifier.is_none()
                                    || specifier == Some("references".to_owned())
                                    || specifier == Some("refs".to_owned())
                                {
                                    println!(
                                        "
                                        Attribute Format Explained:
                                        All Attributes In A Table Definition Must Be Of The Form:
                                        '[key definition][attribute name][attribute type][foreign table]'
                                        Where [key definition] And [foreign table] Can Be Null

                                        If [key definition] Is A Foreign Key [FK | PK/FK | AK/FK]:
                                            Then [foreign table] MUST be defined

                                        The Following Is An Example PK/FK Attribute
                                        Example References A Table Named 'profile' And Its Attribute 'userID'
                                        Attribute -> 'PK/FK userID1 INTEGER profile(userID)'
                                        The Example Would Assign As So:
                                            [key definition] = 'PK/FK'
                                                Is A Primary Key That References Primary Key 'userID' From Table 'profile'
                                            [attribute name] = 'userID1'
                                                Self Explanatory, It Is The Name Of The Attribute Being Defined
                                            [attribute type] = 'INTEGER' 
                                                Type Definitions On Each Attribute Must Match SQL DataTypes, Not Pythons
                                            [foreign table] = 'profile(userID)' Where:
                                                [*] 'profile' Is The Table Being Referenced
                                                [*] 'userID' Is The Referenced Attribute
                                        [*] For More Help With Attributes, Type 'show examples attributes'

                                        In Tables With More Than One PK, It Will Generate Data As A Composite Pair,
                                        Hence The AK Attribute May Be Needed, This Is Enforces The Unique Value Without
                                        Worrying About Checking The Keys As Pairs

                                        In Cases With Tables Having Multiple Attributes: 
                                        They Must Be Placed In () And Separated By Commas.
                                        'show examples add' Provides Examples
                                        "
                                    );
                                }
                                // Explains Attribute Formatting To Aid In Table Input
                                if specifier.is_none()
                                    || specifier == Some("attributes".to_owned())
                                    || specifier == Some("attr".to_owned())
                                {
                                    println!(
                                        "
                                            Basic Introductory To Attribute Types For SQL Tables
                                        ---------------------------------------------------------------------------
                                        [*] To See Custom Defined Types Specifically For This Program
                                        [*] Type 'show examples types'

                                        [Example 1]: 'age INTEGER'
                                        [key definition] = None
                                            [*] No Key Definition is Included
                                        [attribute name] = 'age'
                                            [*] Name of Attribute being Defined
                                        [attribute type] = 'INTEGER'
                                            [*] Must Be SQL Data Types Not Rust Data Types

                                        [Example 2]: 'salary DECIMAL(10,2)'
                                        [key definition] = None
                                            [*] 'No Key Definition is Included
                                        [attribute name] = 'salary'
                                            [*] Name of Attribute being Defined
                                        [attribute type] = 'DECIMAL(10,2)'
                                            [*] Must Be SQL Data Types Not Rust Data Types

                                        [Example 3]: 'name NAME'
                                        [key definition] = None
                                            [*] No Key Definition is Included
                                        [attribute name] = 'orderID'
                                            [*] Name of Attribute being Defined
                                        [attribute type] = 'NAME'
                                            [*] Must Be SQL Data Types Not Rust Data Types
                                            [*] NAME Is A Custom Type Made Specifically For This Program
                                            [*] NAME Type Ensures A Realistic Full Name Is Generated
                                            [*] To See Custom Defined Types Specifically For This Program
                                            [*] Type 'show examples types'

                                        [Example 4]: 'PK userID INTEGER'
                                        [*] Is a Primary Key Attribute
                                        [key definition] = 'PK'
                                            [*] 'PK' stands for Primary Key
                                        [attribute name] = 'userID'
                                            [*] Name of Attribute being Defined
                                        [attribute type] = 'INTEGER'
                                            [*] Must Be SQL Data Types, Not Rust Data Types

                                        [Example 5]: 'AK email EMAIL'
                                        [*] Is a Alternate Key Attribute
                                        [key definition] = 'AK'
                                            [*] 'AK' stands for Alternate Key
                                        [attribute name] = 'userID'
                                            [*] Name of Attribute being Defined
                                        [attribute type] = 'EMAIL'
                                            [*] Must Be SQL Data Types, Not Rust Data Types
                                            [*] EMAIL Is A Custom Type Made Specifically For This Program
                                            [*] EMAIL Type Ensures A Realistic Email Is Generated
                                            [*] If A Name Attribute Is Present In The Same Table And Data Gets Generated For It Before Email
                                            [*] The Email Generated Will Be Representative Of The Name Generated As Well
                                            [*] Example If A Name 'Bob Johnson' Is Generated For A Tuple:
                                            [*] Then A Possible Email Generated Would Be BobJohnson@outlook.com
                                            [*] To See Custom Defined Types Specifically For This Program
                                            [*] Type 'show examples types'

                                        [Example 6]: 'FK customerID INTEGER customers(customerID)'
                                        [*] Is a Foreign Key Attribute Referencing the 'customers' Table's 'customerID' Attribute
                                        [key definition] = 'FK'
                                            [*] 'FK' stands for Foreign Key
                                        [attribute name] = 'customerID'
                                            [*] Name of Attribute being Defined
                                        [attribute type] = 'INTEGER'
                                            [*] Must Be SQL Data Types, Not Rust Data Types

                                        [Example 7]: 'AK/FK customerID INTEGER references customers(customerID)'
                                        [*] Is an Alternate Key and Foreign Key Attribute References the 'customers' Table's 'customerID' Attribute
                                        [key definition] = 'AK/FK'
                                            [*] 'AK/FK' stands for Alternate Key/Foreign Key
                                        [attribute name] = 'customerID'
                                            [*] Name of Attribute Being Defined
                                        [attribute type] = 'INTEGER'
                                            [*] Must be SQL Data Types, not Rust Data Types

                                        [Example 8]: 'PK/FK bookID INTEGER authors(bookID)'
                                        [*] Is a Primary Key and Foreign Key Attribute Referencing the 'authors' Table's 'bookID' Attribute
                                        [key definition] = 'PK/FK'
                                            [*] 'PK' stands for Primary Key
                                            [*] 'FK' stands for Foreign Key
                                            [*] 'PK/FK' stands for Primary Key with a Foreign Reference
                                        [attribute name] = 'bookID'
                                            [*] Name of Attribute being Defined
                                        [attribute type] = 'INTEGER'
                                            [*] Must Be SQL Data Types, Not Rust Data Types
                                        [foreign table] = 'authors(bookID)'
                                            [*] Where Again, 'authors' Is The Table Being Referenced and 'bookID' Is The Referenced Attribute From The Table

                                        [Example 9]: 'PK userID INTEGER, PK postID INTEGER
                                        [*] These Primary Key Attributes Create A Composite Key
                                        [key definition] = 'PK'
                                            [*] 'PK' stands for Primary Key
                                        [attribute name 1] = 'userID'
                                            [*] Name of the First Attribute Being Defined
                                        [attribute type 1] = 'INTEGER'
                                            [*] Must be SQL Data Types, Not Rust Data Types
                                        [attribute name 2] = 'postID'
                                            [*] Name of the second Attribute Being Defined
                                        [attribute type 2] = 'INTEGER'
                                            [*] Must be SQL Data Types, not Rust Data Types
                                        [*] Both 'userID' and 'postID' are Primary Keys
                                        [*] This Means Their Combination Will Be Treated As A Composite Key Pair When Generating Random Data For The Table. 
                                        [*] This Means That Any Two Rows On The Table Cannot Have The Same Combination of 'userID' and 'postID'.

                                        [Example 10]: 'PK/FK bookID INTEGER authors(bookID), PK/FK publisherID INTEGER publishers(publisherID)'
                                        [*] These Primary Key Attributes Create A Composite Key Where Each Attribute Also References An Exterior Table
                                        [key definition] = 'PK/FK'
                                            [*] 'PK' stands for Primary Key
                                            [*] 'FK' stands for Foreign Key
                                            [*] 'PK/FK' stands for Primary Key with a Foreign Reference
                                        [attribute name 1] = 'bookID'
                                            [*] Name of the First Attribute being Defined
                                        [attribute type 1] = 'INTEGER'
                                            [*] Must Be SQL Data Types, Not Rust Data Types
                                        [attribute name 2] = 'publisherID'
                                            [*] Name of the Second Attribute being Defined
                                        [attribute type 2] = 'INTEGER'
                                            [*] Must Be SQL Data Types, Not Rust Data Types
                                        [foreign table] = 'authors(bookID), publishers(publisherID)'
                                            [*] Where Again, 'authors' Is The Table Being Referenced and 'bookID' Is The Referenced Attribute From The Table
                                            [*] As Well As, 'publisher' Is The Table Being Referenced and 'publisherID' Is The Referenced Attribute From The Table
                                            [*] Both 'userID' and 'postID' are Primary Keys
                                            [*] This Means Their Combination Will Be Treated As A Composite Key Pair When Generating Random Data For The Table. 
                                            [*] This Means That Any Two Rows On The Table Cannot Have The Same Combination of 'userID' and 'postID'.

                                        [Example 11]: 'PK userID INTEGER, AK email EMAIL'
                                        [key definition 1] = 'PK'
                                            [*] 'PK' stands for Primary Key
                                        [attribute name 1] = 'userID'
                                            [*] Name of Attribute Being Defined
                                        [attribute type 1] = 'INTEGER'
                                            [*] Must be SQL Data Types, not Rust Data Types
                                        [key definition 2] = 'AK'
                                            [*] 'AK' stands for Alternate Key
                                        [attribute name 2] = 'email'
                                            [*] Name of Attribute Being Defined
                                        [attribute type 2] = 'EMAIL'
                                            [*] Must be SQL Data Types, not Rust Data Types
                                            [*] EMAIL Is A Custom Type Made Specifically For This Program
                                            [*] EMAIL Type Ensures A Realistic Email Is Generated
                                            [*] To See Custom Defined Types Specifically For This Program
                                            [*] Type 'show examples types'
                                            [*] In This table, The 'userID' Attribute Is A Primary Key And The 'email' Attribute Is An Alternate Key.
                                            [*] While Having Multiple Primary Keys In A Table Leads To A Composite Key Pair.
                                            [*] Having A Primary Key And An Alternate Key In The Same Table Doesn't Necessarily Result In A Composite Key.
                                            [*] The Alternate Key May Still Provide A Unique Constraint, But It Doesn't Have The Same Significance As The Primary Key.
                                        "
                                    );
                                }
                            }
                            _ => {
                                println!("Invalid Argument For Show Command");
                                println!("Usage: Show [inserts | keys | references | examples [optional - [add | del | (modify | mod) | (refs | references) | (attributes | attr)]]");
                            }
                        }
                    }
                    _ => {
                        println!("Invalid Command Entered");
                        println!("Type 'Help' For List of Commands");
                    }
                }
            }
        }
    }
}

fn display_help(display_all: bool) {
    /*
     * Displays the help menu
     * If display_all is true, display the entire help menu
     * If display_all is false, display the help menu for the generate command
     */
    println!(
        "
                        ===== MESSAGE =====

            [*]This Program Takes User Input For SQL Table Values And Generates Random Insert Statements For Database Testing
            [*]All Tables Should Be Defined Using The Following Format:
                    Format: [Number Of Insert Statements][Table Name][Table Attributes]
            [*]An Example Of An Accepted Table Definition And An Accepted Add Command:
            [*]add 100 profile (PK userID INTEGER, name NAME, AK email EMAIL, password PASSWORD(30))
            [*]Where 'add' Adds The [Number Of Insert Statements][Table Name][Table Attributes] To A List Used For Generation
            [*]This Program Can Generate Any Number Of Inserts For Any Number Of SQL Tables As Long As User Memory Permits
            [*]The Commands For This Program Are Not Case Sensitive
            [*]Type 'Help' To Get A List Of All Commands And To Reprint This Message
            [*]To See Example Commands For Each Command:
                    Type 'show examples [specific command (optional - will show all examples if blank)]'
            [*]Type 'Clear' To Delete This Message
            \n"
    );

    if display_all {
        println!(
            "
                        ===== COMMANDS AND DEFINITIONS=====

            Commands Followed By [] Are Required Args Unless Specifically Stated In This Menu

            Generate -> Begins Generating SQL Script Of Inserts [Mut Have At Least One Defined Table]
            Clear -> Clear Terminal Screen
            Help -> Show This Help Menu
            Exit | Quit -> Terminate Program

            Add  [numInserts][tableName][tableAttributes] -> Add Table To Generate Statements For, Where:
                 [numInserts] -> The Number Of Insert Statements To Generate For The Table
                 [tableName] -> The Name of The Table Being Created
                 [tableAttributes] -> All Defined Columns Of The Table [Format Shown Below]
                 [tableAttributes] Should Be Input In The Following Format:

                 [key definition][referenced_attr name][referenced_attr type][foreign table]

                 Where [key definition] and [foreign table] can be NULL
                 Key Definitions:
                    'PK' - Primary Key
                    'AK' - Alternate (Unique) Key
                    'FK' - Foreign Key -> [foreign table] - Table Being Referenced By The Attribute
                    'PK/FK' - Primary Key And Foreign Key -> [foreign table] - Table Being Referenced By The Attribute
                    'AK/FK' - Alt (Unique) And Foreign Key -> [foreign table] - Table Being Referenced By The Attribute

                 [Note 1]: In Cases Of An Attribute Being A Foreign Key, [foreign table] CANNOT be NULL
                 To Ensure A Foreign Key Attribute Is Properly Read And Referenced A Specific Format Is Provided:
                 [Foreign Table] Proper Format -> referenced_table(referenced_attribute)
                 For More Info On Referencing Syntax With Examples. Input 'show examples [refs | references]'

                 [Note 2]: In Tables With More Than One PK, It Will Generate Data As A Composite Pair
                 Hence The AK Attribute May Be Needed, This Enforces The Unique Values Without Worrying About Checking The Keys As Pairs

            Rm   [tableName] -> Remove Table From List
                 [tableName] Must Be The Same As It Was Defined In It's Add Statement

            Modify | Mod [tableName] [numStatements | tableName | referenced_attr] [newValue] -> Modify Existing Table
                 [tableName] -> The Table To Modify And Must Be The Same As It Was Defined In It's Add Statement
                 [numInserts | tableName | referenced_attr] -> The Definition Of The Tuple You Wish To Modify 
                 Note: You Can Only Modify One Definition At A Time
                 IMPORTANT REGARDING Modify [numInserts | tableName]:
                 If User Wishes To Modify The Number of Statements or Change Table Name. The User Should Input 'numStatements'
                 or 'tableName' in Place of First Arg After Specifying The Table: However, When Modifying [referenced_attr],
                 The User Should Input the Attribute Name They Wish To Change. Type 'show examples mod' For More Info
                 [*]For 'Modify [tableName] [referenced_attr]':
                    The User Must Redefine The Whole Attribute 
                    As In Redefine [key definition][referenced_attr name][referenced_attr type][foreign table]

            Show [inserts | keys | references | types | examples [Add | Del | (Modify | Mod) | (Refs | References) | (Attributes | Attr)]
                 [Inserts] -> Show's The Table's The Program Will Be Creating Insert Statements For
                 [Keys] -> Show's The List Of Keys For Each Table
                 [References | Refs] -> Show's All Referenced Attributes Between Two Tables In The Form Of:
                                        ([Referencing Table]:([Referenced Table]:[Referenced Attribute]))
                 [Types] -> Lists All Custom Defined Types And The Reasoning For Their Creation For This Program 
                 [Examples] -> Will Print Examples For Most Commands Taken By The Program
            "
        );
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn test_attribute_datatype() {
        /*
        Test Valid and Invalid Data Types
        Assert Valid Data Types Return True
        Assert Invalid Data Types Return False
        */

        let valid_datatypes = vec!["INTEGER", "VARCHAR(30)", "PASSWORD(30)"];
        let invalid_datatypes = vec!["INVALID!", "VAFCHAR", "EMMAIL"];
        for valid in valid_datatypes {
            assert_eq!(super::check_data_type(valid), true);
        }
        for invalid in invalid_datatypes {
            assert_eq!(super::check_data_type(invalid), false);
        }
    }

    #[test]
    fn test_key_definitions() {
        /*
        Test Valid and Invalid Key Definitions
        For Valid, Test All Possible Key Definitions
        For Invalid, Test Spelling Errors and Incorrect Ordering
        */
        let valid_key_definitions = vec!["PK", "AK", "FK", "PK/FK", "AK/FK"];
        let invalid_key_definitions = vec!["KP", "KA", "KF", "PK/AK", "AK/PK", "FK/PK", "FK/AK", "PK/AK/FK"];
        for valid in valid_key_definitions {
            assert_eq!(super::check_key_definition(valid), true);
        }
        for invalid in invalid_key_definitions {
            assert_eq!(super::check_key_definition(invalid), false);
        }
    }

    #[test]
    fn test_random_data_generation() {
        /*
        Test Data Generation With Types With Given Size
        PASSWORD(20), USERNAME(10), MONEY(7), VARCHAR(30)
        Assert Length of Return > 0 and <= Given Size
        */
        let statement_data: HashMap<String, String> = HashMap::new(); //Not Needed For This Test (Only Used For Email Generation)
        
        let password = super::get_random_data("PASSWORD(20)", Some(20), &statement_data);
        assert_eq!(password.len() > 0 && password.len() <= 20, true);

        let username = super::get_random_data("USERNAME(10)", Some(10), &statement_data);
        assert_eq!(username.len() > 0 && username.len() <= 10, true);

        let money = super::get_random_data("MONEY(7)", Some(7), &statement_data);
        //Since money is returned as String and .{}{} (Used for cents) takes up 3 chars. The length of the return should be 7 + 3 = 10
        assert_eq!(money.len() > 0 && money.len() <= 10, true);

        let varchar = super::get_random_data("VARCHAR(30)", Some(30), &statement_data);
        assert_eq!(varchar.len() > 0 && varchar.len() <= 30, true);
    }

    #[test]
    fn test_email_generation_with_names_in_statement_data() {
        /*
        Test Email Generation With Names In Statement Data
        Assert Email Contains Name
        This Tests The 3 Given Key Possiblities (name, full name, full_name)
        */
        let mut statement_data: HashMap<String, String> = HashMap::new();
        statement_data.insert("name".to_string(), "Bob Johnson".to_string());
        let email = super::get_random_data("EMAIL", None, &statement_data);
        assert_eq!(email.contains("BobJohnson"), true);

        //Reset HashMap and generate new name, then try again
        statement_data = HashMap::new();
        statement_data.insert("full name".to_string(), "John Smith".to_string());
        let email = super::get_random_data("EMAIL", None, &statement_data);
        assert_eq!(email.contains("JohnSmith"), true);

        //Reset HashMap and generate new name, then try again
        statement_data = HashMap::new();
        statement_data.insert("full_name".to_string(), "Jane Doe".to_string());
        let email = super::get_random_data("EMAIL", None, &statement_data);
        assert_eq!(email.contains("JaneDoe"), true);
    }

    #[test]
    fn test_set_variable_size() {
        /*
        Pass datatypes of format DATATYPE(n)
        Assert n is returned
        */
        let data_types = vec!["VARCHAR(30)", "PASSWORD(20)", "USERNAME(10)", "MONEY(7)"];

        //Loop through each data type and assert the size is returned
        for data_type in data_types {
            let size = super::set_variable_size(data_type);
            assert_eq!(size.is_some(), true);
            assert_eq!(size.unwrap() > 0, true);
        }
    }

    #[test]
    fn test_create_insert_statement() {
        /*
        Create vars for function and test returned insert statement
        */
        let target_insert_statement = "INSERT INTO profile VALUES (1, 'Bob Johnson', 'BobJohnson@pitt.edu');";
        let table_name = "profile";
        let table_attributes: Vec<String> = vec!["userID".to_string(), "name".to_string(), "email".to_string()];
        let mut statement_data: HashMap<String, String> = HashMap::new();
        statement_data.insert("userID".to_string(), "1".to_string());
        statement_data.insert("name".to_string(), "Bob Johnson".to_string());
        statement_data.insert("email".to_string(), "BobJohnson@pitt.edu".to_string());
        
        let generated_insert = super::create_insert_statement(table_name, &table_attributes, &statement_data);
        assert_eq!(generated_insert, target_insert_statement);
    }

    #[test]
    fn test_check_pair_with_unique_pair_passed() {
        /*
        Test check_pair with a unique pair
        pair_changed should return false
        Generate values for 
                generated_pair_vector: &Vec<String>,
                previous_pairs: &Vec<Vec<String>>,
                table_attributes: &Vec<String>,
                uq_attributes: &HashMap<String, Vec<String>>,
                count: usize,
        */
        let generated_pair_vector: Vec<String> = vec!["1".to_string(), "Bob Johnson".to_string()];
        let previous_pairs: Vec<Vec<String>> = vec![vec!["2".to_string(), "John Smith".to_string()], vec!["3".to_string(), "Jane Doe".to_string()], vec!["4".to_string(), "Steven Even".to_string()]];
        let table_attributes: Vec<String> = vec!["userID".to_string(), "name".to_string()];
        let mut uq_attributes: HashMap<String, Vec<String>> = HashMap::new();
        uq_attributes.insert("userID".to_string(), vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()]);
        uq_attributes.insert("name".to_string(), vec!["Bob Johnson".to_string(), "John Smith".to_string(), "Jane Doe".to_string(), "Steven Even".to_string()]);
        let count: usize = 0;
        let (pair_changed, _new_pair) = super::check_pair(&generated_pair_vector, &previous_pairs, &table_attributes, &uq_attributes, count);
        assert_eq!(pair_changed, false);
    }

    #[test]
    fn test_check_pair_with_prev_pair_passed() {
        /*
        Test check_pair with a prev generated pair
        pair_changed should return true
        Then check_pair with new values should return false
        Generate values for 
                generated_pair_vector: &Vec<String>,
                previous_pairs: &Vec<Vec<String>>,
                table_attributes: &Vec<String>,
                uq_attributes: &HashMap<String, Vec<String>>,
                count: usize,
        */
        let generated_pair_vector: Vec<String> = vec!["1".to_string(), "Bob Johnson".to_string()];
        let previous_pairs: Vec<Vec<String>> = vec![vec!["2".to_string(), "John Smith".to_string()], vec!["1".to_string(), "Bob Johnson".to_string()], vec!["4".to_string(), "Steven Even".to_string()]];
        let table_attributes: Vec<String> = vec!["userID".to_string(), "name".to_string()];
        let mut uq_attributes: HashMap<String, Vec<String>> = HashMap::new();

        //Add extra padding to both attributes to allow function to generate new pair. Run check pair twice
        uq_attributes.insert("userID".to_string(), vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string()]);
        uq_attributes.insert("name".to_string(), vec!["Bob Johnson".to_string(), "John Smith".to_string(), "Jane Doe".to_string(), "Steven Even".to_string(), "Zoe Tae".to_string(), "Jenny Doe".to_string()]);
        let count: usize = 0;
        let (pair_changed, new_pair) = super::check_pair(&generated_pair_vector, &previous_pairs, &table_attributes, &uq_attributes, count);
        assert_eq!(pair_changed, true);
        let (pair_changed, _new_pair) = super::check_pair(&new_pair, &previous_pairs, &table_attributes, &uq_attributes, count);
        assert_eq!(pair_changed, false);
    }
}
