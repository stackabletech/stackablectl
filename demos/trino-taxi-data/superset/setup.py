import logging
import requests

base_url = "http://superset:8088"
# base_url = "http://172.18.0.4:31024"
username = "admin"
password = "admin"

logging.basicConfig(level=logging.INFO)
logging.info("Starting setup of Superset")

logging.info("Getting access token from /api/v1/security/login")
session = requests.session()
access_token = session.post(f"{base_url}/api/v1/security/login", json={"username": username, "password": password, "provider": "db", "refresh": True}).json()['access_token']
# print(f"access_token: {access_token}")

logging.info("Getting csrf token from /api/v1/security/csrf_token")
csrf_token = session.get(f"{base_url}/api/v1/security/csrf_token", headers={"Authorization": f"Bearer {access_token}"}).json()["result"]
# print(f"csrf_token: {csrf_token}")

headers = {
    "accept": "application/json",
    "Authorization": f"Bearer {access_token}",
    "X-CSRFToken": csrf_token,
}

# logging.info("Exporting all assets")
# result = session.get(f"{base_url}/api/v1/assets/export", headers=headers)
# assert result.status_code == 200
# with open("assets.zip", "wb") as f:
#     f.write(result.content)


#########################
# IMPORTANT
#########################
# The exported file had to be modified, otherwise we get:
# <Response [422]>
# {"errors": [{"message": "Error importing assets", "error_type": "GENERIC_COMMAND_ERROR", "level": "warning", "extra": {"databases/Trino.yaml": {"extra": {"disable_data_preview": ["Unknown field."]}}, "issue_codes": [{"code": 1010, "message": "Issue 1010 - Superset encountered an error while running a command."}]}}]}
#
# The file databases/Trino.yaml was modified and the attribute "extra.disable_data_preview" was removed
#########################
logging.info("Importing all assets")
files = {
    "bundle": ("assets.zip", open("assets.zip", "rb")),
}
data = {
    "passwords": '{"databases/Trino.yaml": "demo"}'
}
result = session.post(f"{base_url}/api/v1/assets/import", headers=headers, files=files, json=data)
print(result)
print(result.text)
assert result.status_code == 200

logging.info("Finished setup of Superset")
