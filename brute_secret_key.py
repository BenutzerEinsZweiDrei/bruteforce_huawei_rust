import hashlib
import itertools

def generate_secret_key(length):
    return ''.join(itertools.islice(itertools.cycle('0123456789abcdef'), length))

def brute_force_secret_key(imei, sn, pid, hw_id, target):
    for length in range(1, 17):  # Assuming the secret key length is between 1 and 16 characters
        for secret_key in itertools.product('0123456789abcdef', repeat=length):
            print(secret_key)
            secret_key = ''.join(secret_key)
            concatenated_string = f"{imei}{sn}{pid}{secret_key}{hw_id}"
            hash_object = hashlib.sha256(concatenated_string.encode())
            hex_dig = hash_object.hexdigest()
            truncated_hash = hex_dig[:16]  # Truncate to 16 characters
            if truncated_hash == target:
                return secret_key
    return None

# Example usage
imei = "866630026707460"
sn = "W3D7N"
pid = "155"
hw_id = "30000759"
target = "1271121224224792"  # This should be the truncated hash in hexadecimal format

secret_key = brute_force_secret_key(imei, sn, pid, hw_id, target)
if secret_key:
    print(f"Secret key found: {secret_key}")
else:
    print("Secret key not found")
