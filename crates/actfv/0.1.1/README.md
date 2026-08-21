# actfv

Take `.aws/credentials` as input and write it into a Terraform compatible `.tfvars` file.

In some work scenarios, the aws credentials time out in a very short amount of time. This forces a new aws login, and
then those credentials need to make it into terraform.

This utility allows running terraform and passing in a --var-file that contains these secrets. Make sure this file is
outside of your version control or add it to .gitignore so you don't accidentally commit your aws credential 
information.


It will find and print out the following:

- region
- aws_access_key_id
- aws_secret_access_key
- aws_session_token

## Usage

`actfv <aws_credentials_file> <output_tfvars_file> <profile>`

- aws_credentials_file - Typically found in `~/.aws/credentials`
- output_tfvars_file - The file to be written out. Ex: `secrets.tfvars`
- profile - Many times this is just `default`, but work scenarios may often include `adfs`