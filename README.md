# randomSQL
Rust program that generates mock data and uses created data to generate SQL insert statements. This program maintains key constraints and is able to handle referenced attributes from other tables. You should be able to input your entire schema into the program and it will generate and cross reference values for each table.

# Example Schema
table 'profile' (
  userID INT NOT NULL,
  name VARCHAR(50) NOT NULL,
  email VARCHAR(30) UNIQUE NOT NULL,
  password VARCHAR(30) NOT NULL,
  dateOfBirth DATE NOT NULL,
  PRIMARY KEY (userID)
)

table 'friend' (
  friend1 INT NOT NULL,
  friend2 INT NOT NULL,
  friendDate DATE NOT NULL,
  PRIMARY KEY (friend1, friend2),
  FOREIGN KEY (friend1) REFERENCES profile(userID) ON DELETE CASCADE,
  FOREIGN KEY (friend2) REFERENCES profile(userID) ON DELETE CASCADE
)

# Translating Schema to Add Command in RandomSQL
For 'profile' We will generate 150 profiles

'add 150 profile (PK userID INTEGER, name NAME, AK email EMAIL, password PASSWORD(30), dateOfBirth DATE)'

For 'friend' We will generate 240 friendships

'add 240 friend (PK/FK friend1 INTEGER profile(userID), PK/FK friend2 INTEGER profile(userID), friendDate DATE)'

This will generate 150 unique profiles and 240 unique friendships using data (friend1 and friend2) created from the profile command
(i.e All friend1 and friend2 userIDs will be userIDs created during the creation of the 150 profile statements)
Follow this syntax when translating for any schema, as long as the type has support for random data generation (see generate_random_data function for all supported types as of now)
Then it will be able to create any number of insert statements for that schema (As long as composite keys that reference from another table can mathematically work)
(i.e if only 4 profiles are generated, then it is impossible to create 7 or more friendships due to the uniqueness of combinations which will cause a stack overflow from recursion of check_pair)
