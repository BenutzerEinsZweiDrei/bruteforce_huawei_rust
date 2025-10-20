import numpy as np

def digit_sum(num):
    return sum(int(d) for d in str(num))

def find_alpha_beta_delta_multiple_c(a, b, c_list, digit_sum_target=84, mod=10**16, search_limit=20, filename="solutions.txt"):
    a_mod = a % mod
    b_mod = b % mod

    candidates = np.arange(-search_limit, search_limit+1)
    alphas, betas, deltas = np.meshgrid(candidates, candidates, candidates, indexing='ij')
    alphas = alphas.ravel()
    betas = betas.ravel()
    deltas = deltas.ravel()

    total_solutions = 0
    with open(filename, 'w') as f:
        for idx, c in enumerate(c_list):
            c_mod = c % mod
            d_values = (a_mod * alphas + b_mod * betas + c_mod * deltas) % mod
            
            mask_16_digits = (d_values >= 10**15)
            
            filtered_d = d_values[mask_16_digits]
            filtered_alpha = alphas[mask_16_digits]
            filtered_beta = betas[mask_16_digits]
            filtered_delta = deltas[mask_16_digits]

            for i, d in enumerate(filtered_d):
                if digit_sum(d) == digit_sum_target:
                    alpha = filtered_alpha[i]
                    beta = filtered_beta[i]
                    delta = filtered_delta[i]
                    # Save format: c_variant_index alpha beta delta d sum_of_digits
                    f.write(f"{d}\n")
                    total_solutions += 1
    
    print(f"Checked {len(c_list)} variants of c.")
    print(f"Found {total_solutions} solutions total. Saved to {filename}")

# Example usage:
prefix = 1
serial = 1
imei = 1
a = imei
b = int(prefix, 36) * serial
c_variants = [int("HUAWEI2018", 36), int("HUAWEI2016", 36), int("HUAWEI2017", 36), int("HUAWEI2015", 36)]  # example list of c variants

find_alpha_beta_delta_multiple_c(a, b, c_variants, digit_sum_target=84, search_limit=20)
