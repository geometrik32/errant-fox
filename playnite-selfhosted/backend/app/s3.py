import boto3
from botocore.config import Config
from botocore.exceptions import ClientError
from fastapi import HTTPException
from app.config import settings

def get_s3_client():
    endpoint = f"http://{settings.minio_endpoint}"
    if settings.minio_secure:
        endpoint = f"https://{settings.minio_endpoint}"

    return boto3.client(
        "s3",
        endpoint_url=endpoint,
        aws_access_key_id=settings.minio_access_key,
        aws_secret_access_key=settings.minio_secret_key,
        config=Config(signature_version="s3v4"),
        verify=settings.minio_secure
    )

def ensure_buckets_exist():
    s3 = get_s3_client()
    for bucket in [settings.bucket_games, settings.bucket_saves]:
        try:
            s3.head_bucket(Bucket=bucket)
        except ClientError:
            try:
                s3.create_bucket(Bucket=bucket)
                print(f"Bucket '{bucket}' created successfully.")
            except Exception as e:
                print(f"Failed to create bucket '{bucket}': {e}")

def get_presigned_download_url(bucket: str, key: str, expires_in: int = 3600) -> str:
    s3 = get_s3_client()
    try:
        url = s3.generate_presigned_url(
            "get_object",
            Params={"Bucket": bucket, "Key": key},
            ExpiresIn=expires_in
        )
        return url
    except ClientError as e:
        raise HTTPException(status_code=500, detail=f"S3 client error: {e}")

def get_presigned_upload_url(bucket: str, key: str, expires_in: int = 3600) -> str:
    s3 = get_s3_client()
    try:
        url = s3.generate_presigned_url(
            "put_object",
            Params={"Bucket": bucket, "Key": key},
            ExpiresIn=expires_in
        )
        return url
    except ClientError as e:
        raise HTTPException(status_code=500, detail=f"S3 client error: {e}")
