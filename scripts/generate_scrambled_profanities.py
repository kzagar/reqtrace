#!/usr/bin/env python3
"""
generate_scrambled_profanities.py

Generates all possible 5-letter proquints (65,536 standard CVCVC proquints),
identifies those that match obscene / profane words from online sources
(English, Slovenian, German, Spanish, and French), merges directly curated
scrambled tokens, and scrambles matching proquints using XOR key 3.

Outputs a sorted list of unique scrambled words, one word per line.
"""

import json
import subprocess
import sys
import unicodedata
from typing import Set

CONSONANTS = ['b', 'd', 'f', 'g', 'h', 'j', 'k', 'l', 'm', 'n', 'p', 'r', 's', 't', 'v', 'z']
VOWELS = ['a', 'i', 'o', 'u']
XOR_KEY = 3

LANGUAGES = ['en', 'sl', 'de', 'es', 'fr']

# Directly curated scrambled tokens (XOR key 3) for terms without standard online list sources
# e.g., Slovenian proquints:
#   'fukat' -> 'evhbw', 'fukal' -> 'evhbo', 'fukan' -> 'evhbm',
#   'riton' -> 'qjwlm', 'lulat' -> 'ovobw', 'kakat' -> 'hbhbw', 'kalan' -> 'hbobm'
DIRECT_SCRAMBLED_TOKENS = {
    "evhbw", "evhbo", "evhbm", "qjwlm", "ovobw", "hbhbw", "hbobm"
}


def fetch_url(url: str) -> str:
    """Fetch URL content using curl with a timeout."""
    cmd = ["curl", "-fsSL", "--max-time", "15", url]
    res = subprocess.run(cmd, capture_output=True, text=True, check=True)
    return res.stdout


def load_online_profane_words() -> Set[str]:
    raw_words: Set[str] = set()

    # 1. Swearify multi-language repository
    try:
        content = fetch_url('https://raw.githubusercontent.com/Behiwzad/swearify/master/data/words.json')
        sw = json.loads(content)
        for lang in LANGUAGES:
            raw_words.update(sw.get(lang, []))
    except Exception as e:
        print(f"[!] Warning fetching Swearify via curl: {e}", file=sys.stderr)

    # 2. LDNOOBW lists
    for lang in ['en', 'de', 'es', 'fr']:
        try:
            url = f'https://raw.githubusercontent.com/LDNOOBW/List-of-Dirty-Naughty-Obscene-and-Otherwise-Bad-Words/master/{lang}'
            content = fetch_url(url)
            raw_words.update(line.strip() for line in content.splitlines() if line.strip())
        except Exception as e:
            print(f"[!] Warning fetching LDNOOBW {lang} via curl: {e}", file=sys.stderr)

    # 3. ValentinH lists
    for lang in ['en', 'de', 'es', 'fr']:
        try:
            url = f'https://raw.githubusercontent.com/ValentinH/swear-words/master/{lang}'
            content = fetch_url(url)
            raw_words.update(line.strip() for line in content.splitlines() if line.strip())
        except Exception as e:
            print(f"[!] Warning fetching ValentinH {lang} via curl: {e}", file=sys.stderr)

    # 4. bunsogs multi-language filter
    try:
        url = 'https://raw.githubusercontent.com/VityaSchel/bunsogs-profanity-filter/main/src/profanity-list.json'
        content = fetch_url(url)
        bs = json.loads(content)
        for lang in LANGUAGES:
            raw_words.update(bs.get(lang, []))
    except Exception as e:
        print(f"[!] Warning fetching Bunsogs via curl: {e}", file=sys.stderr)

    # Clean & normalize (lower-case, remove comments, strip accents)
    cleaned = set()
    for w in raw_words:
        w_norm = w.strip().lower()
        if not w_norm or w_norm.startswith('#'):
            continue
        cleaned.add(w_norm)
        w_ascii = ''.join(c for c in unicodedata.normalize('NFD', w_norm) if unicodedata.category(c) != 'Mn')
        if w_ascii != w_norm:
            cleaned.add(w_ascii)

    return cleaned


def main():
    output_file = sys.argv[1] if len(sys.argv) > 1 else "scrambled_profanities.txt"

    proquints = {
        f"{c1}{v1}{c2}{v2}{c3}"
        for c1 in CONSONANTS
        for v1 in VOWELS
        for c2 in CONSONANTS
        for v2 in VOWELS
        for c3 in CONSONANTS
    }

    profane_words = load_online_profane_words()
    flagged_proquints = proquints.intersection(profane_words)

    # Scramble online profane matches with XOR constant 3
    scrambled = {
        "".join(chr(ord(c) ^ XOR_KEY) for c in word)
        for word in flagged_proquints
    }

    # Add directly curated scrambled tokens
    scrambled.update(DIRECT_SCRAMBLED_TOKENS)

    sorted_scrambled = sorted(scrambled)

    with open(output_file, "w", encoding="utf-8") as f:
        for s in sorted_scrambled:
            f.write(f"{s}\n")

    print(f"Generated {len(sorted_scrambled)} scrambled profane words in {output_file}", file=sys.stderr)


if __name__ == "__main__":
    main()
