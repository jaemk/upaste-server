#!/usr/bin/python
# -*- coding: utf-8 -*-

from __future__ import unicode_literals
from __future__ import print_function

try:
    from urllib.request import urlopen
except ImportError:
    from urllib2 import urlopen

import os
import sys
import json
import tarfile


BASE_DIR    = os.path.dirname(__file__)
BIN_DIR     = os.path.join(BASE_DIR, 'bin')
CONFIG_FILE = os.path.join(BASE_DIR, '.update')
REPO        = 'upaste-server'


def dl_progress(bytes_read, file_size):
    """
    Display download progress bar. If no file_size is specified. default to 100%
    """
    if file_size:
        ratio = float(bytes_read) / file_size
    else:
        ratio = 1
    percent = int(ratio * 100)

    bar_len = 60
    done = int(bar_len * ratio)
    bar = ('=' * done) + (' ' * (bar_len - done))

    progress = '{percent: >3}%: [{bar}]'.format(percent=percent, bar=bar)
    backspace = '\b' * len(progress)
    print(backspace + '\r', end='')
    print(progress, end='')


def get_content_length(headers):
    """
    python2 names headers all lowercase, python3 has them title-case
    """
    ctl = 'content-length'
    for k, v in headers.items():
        if k.lower() == ctl:
            return int(v)
    return None


def download_to_file(url, filename):
    """
    Download the given url to specified filename
    """
    resp = urlopen(url)
    file_size = get_content_length(resp.headers)
    block_size = 8192
    bytes_read = 0
    with open(filename, 'wb') as f:
        while True:
            buf = resp.read(block_size)
            if not buf:
                break
            bytes_read += len(buf)
            f.write(buf)
            dl_progress(bytes_read, file_size)
    print(' ✓')


def get_input(prompt):
    """
    Get user input 2/3 agnostic
    """
    try:
        try:
            return raw_input(prompt)
        except NameError:
            return input(prompt)
    except EOFError:
        return ''


def main():
    # get latest git release information
    latest = urlopen("https://api.github.com/repos/jaemk/{}/releases/latest".format(REPO))
    latest = json.loads(latest.read().decode('utf-8'))
    latest_tag = latest['tag_name']

    # look for config file with current tag and target-triple
    config = {'tag': 'none', 'target': None}
    if os.path.isfile(CONFIG_FILE):
        with open(CONFIG_FILE, 'r') as f:
            config = json.load(f)

    print("Current Tag: {}".format(config['tag']))
    print("Latest  Tag: {}".format(latest_tag))

    if config['tag'] == latest_tag:
        print("** upaste is up-to-date!")
        return


    # get info on files available for download in the latest release
    bins_info = latest['assets']
    bins = [{'name': b['name'], 'download': b['browser_download_url']} for b in bins_info]
    n_bins = len(bins)

    target = config.get('target', None)
    if not target:
        # ask which binary to download
        while True:
            print("\nAvailable binaries:")
            for i, b in enumerate(bins):
                print("  {}: {}".format(i, b['name']))
            n = get_input("\nPlease enter the key of the binary to download >> ")
            try:
                n = int(n)
                if n < n_bins:
                    break
                else:
                    print("\nError: Key `{}` out of range `{}-{}`".format(n, 0, n_bins - 1))
            except ValueError:
                print("\nError: Please enter a number")
        new_bin = bins[n]

        # ex. upaste-v0.2.4-x86_64-unknown-linux-gnu.tar.gz
        target = new_bin['name'].rstrip('.tar.gz').split('-')[2:]
        target = '-'.join(target)
    else:
        new_bin = None
        for b in bins:
            if target in b['name']:
                new_bin = b
                break
        if new_bin is None:
            print("Error: target triple `{}` saved in `{}` not found in available releases:".format(target, os.path.basename(CONFIG_FILE)))
            for b in bins:
                print("  {}".format(b['name']))

    # download binary tarball
    print("** fetching `{}`".format(new_bin['name']))
    download_to_file(new_bin['download'], new_bin['name'])

    # extract binary
    print("** extracting binary to `bin/upaste`")
    tar = tarfile.open(new_bin['name'], 'r:gz')
    tar.extractall()
    tar.close()
    os.system('mkdir -p {}'.format(BIN_DIR))
    os.system('mv upaste bin')

    # delete tarball
    print("** cleaning up `{}`".format(new_bin['name']))
    os.remove(new_bin['name'])

    # update local release tag / target in our config
    config['tag']    = latest_tag
    config['target'] = target
    print("** updating local tag in `.update`")
    with open(CONFIG_FILE, 'w') as f:
        f.write(json.dumps(config, sort_keys=True, indent=4))

    # update project files
    print("** updating project files")
    os.system('git pull')


if __name__ == '__main__':
    try:
        main()
    except KeyboardInterrupt:
        print("\nExiting...")

