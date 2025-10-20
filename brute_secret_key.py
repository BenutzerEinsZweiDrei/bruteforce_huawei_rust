import hashlib
import itertools
import multiprocessing
import time

def brute_force_secret_key(imei, sn, pid, hw_id, target, update_eta):
    total_keys = sum(16 ** length for length in range(1, 17))
    print(total_keys)
    start_time = time.time()
    processed_keys = 0
    update_interval = total_keys // 100  # Update ETA every 1% of total keys

    for length in range(5, 17):  # Assuming the secret key length is between 1 and 16 characters
        for secret_key in itertools.product('0123456789abcdef', repeat=length):
            print(secret_key)
            secret_key = ''.join(secret_key)
            concatenated_string = f"{imei}{sn}{pid}{secret_key}{hw_id}"
            hash_object = hashlib.sha256(concatenated_string.encode())
            hex_dig = hash_object.hexdigest()
            truncated_hash = hex_dig[:16]  # Truncate to 16 characters
            if truncated_hash == target:
                return secret_key
            processed_keys += 1

    return None

def worker(example, update_eta):
    imei, sn, pid, hw_id, target = example
    return brute_force_secret_key(imei, sn, pid, hw_id, target, update_eta)

def update_eta(eta):
    print(f"ETA: {eta}")

if __name__ == "__main__":
    examples = [
        ("866630026707460", "W3D7N", "155", "30000759", "1271121224224792"),
        ("869851020410516", "GFMDU", "161", "09002672", "7223315747388134"),
        ("865218031601802", "LHTDU", "174", "10006354", "2505513276223133")
    ]

    with multiprocessing.Pool() as pool:
        results = pool.starmap(worker, [(example, update_eta) for example in examples])

    for i, result in enumerate(results):
        if result:
            print(f"Secret key found for example {i + 1}: {result}")
        else:
            print(f"Secret key not found for example {i + 1}")
