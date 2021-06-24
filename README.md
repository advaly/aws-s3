## Summary

Sample code for upload or download a file from AWS S3, using rust-s3 crate.

## How to Build

We have checked with the following toolchain versions.

- cargo 1.53.0-nightly (f3e13226d 2021-04-30)
- rustc 1.54.0-nightly (676ee1472 2021-05-06)


## How to Use

```
USAGE:
    aws-s3 [FLAGS] [OPTIONS] <list|get|put|delete>

FLAGS:
        --debug      Enable debug print
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --access_key_id <access key id>            AWS_ACCESS_KEY_ID
    -b, --bucket <bucket>                          AWS S3 bucket name
    -c, --config <config>                          Config file path [default: aws-s3.json]
    -l, --local <local>                            Local file path to put/get
    -g, --region <region>                          AWS S3 region name
    -r, --remote <remote>                          Remote file path on AWS S3
    -s, --secret_access_key <secret access key>    AWS_SECRET_ACCESS_KEY

ARGS:
    <list>      List objects on remote
    <get>       Get a file from remote
    <put>       Put a file to remote
    <delete>    Delete a file from remote
```

### Set AWS S3 Access Keys

There are two ways to set access keys.
(One more thing...configuration file described later)

#### Environment variables

Set `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` as environment variables.

Example:
```
$ export AWS_ACCESS_KEY_ID=id
$ export AWS_SECRET_ACCESS_KEY=secret
```

#### Pass as command line arguments

Example:
```
$ aws-s3 list --access_key_id=<id> --secret_access_key=<secret> ...
```

### Operation examples

In the following examples, access keys are assumed to be set as envirinment variables.

#### LIST

Output list of objects on the S3 storage to the stdout.

Example:
```
$ aws-s3 list --region=<region> --bucket=<bucket>
```

#### GET

Get a file from S3 strage.

Need to specify local and remote path with command line arguments.

- `--remote`: Target object path to get from the S3 storage
- `--local`: Local path to save the retrieved file
  - If you specify a directory path for `local`, the destination local file name is set to the same name as remote

Example1: Specify directory path for `local`. 'hoge.txt' on the S3 is retrieved as '/tmp/hoge.txt'.
```
$ aws-s3 get --region=<region> --bucket=<bucket> --remote=hoge.txt --local=/tmp
```

Example2: Speficy absolute path for `local`. 'hoge.txt' on the S3 is retrieved as '/tmp/fuga.txt'
```
$ aws-s3 get --region=<region> --bucket=<bucket> --remote=hoge.txt --local=/tmp/fuga.txt
```

#### PUT

Put a file to S3 strage.

Need to specify local and remote path with command line arguments.

- `--remote`: Target object path to put on the S3 storage
- `--local`: Local path for a file to put

Example:
```
$ aws-s3 put --region=<region> --bucket=<bucket> --remote=fuga.txt --local=hoge.txt
```

#### DELETE

Delete a file from S3 storage.

Need to specify a remote path to delete with command line arguments.

- `--remote`: Target object path to delete from the S3 storage

Example:
```
$ aws-s3 delete --region=<region> --bucket=<bucket> --remote=fuga.txt
```

## Configuration File

You could also use a configuration file to abbreviate command line arguments.

Following parameters can be load from a configuration file instead of specifying on the command line.
The default configuration file name is 'aws-s3.json' in the current directory.
This default file name is changed by the command line option `--config`.

When the configuration file found, aws-s3 load followig settings from the configuration file.

- access key id
- secret access key
- bucket
- region
- local

If same parameters are speficied by command line even though the configuration file is loaded,
aws-s3 uses command line arguments first.

The S3 access keys also could be defined as environment variable. So the priorities are as follows.

Command line options > Configuration file > Environment variables

### File format

The configuration file is described in json format.

Example: aws-s3.json
```json
{
    "region": "your region",
    "bucket": "your bucket",
    "access_key_id": "your access key id",
    "secret_access_key": "your secret access key",
    "local": "/tmp"
}
```

You do not need to fill all the value in the configuration file.
For example if you want to set only `region` and `bucket` parameters in the configuration file, you do not need to write other definitions of `access_key_id`, `secret_access_key` and `local`. Leave as blank string "". 
