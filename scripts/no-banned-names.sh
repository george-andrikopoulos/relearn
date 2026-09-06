#!/usr/bin/env bash
# no-banned-names.sh — names that must never enter this repository.
#
# WHY THIS EXISTS (2026-09-06). This repository stores other repositories'
# incidents verbatim. A rule's `incident` field is a quotation from a private
# working session, and `TODO.md` narrates the same material. Both are public.
#
# On 2026-09-06 an employer-owned product name was found in
# `rules/repair-the-lying-artefact.md` and twice in `TODO.md` — carried in all
# 52 commit trees, in TODO.md since the first commit. The source repository
# (Design-Architecture-Tool) has had a working gate for exactly this since its
# M33-A4, and that gate did not help, because a gate belongs to a repository
# and the *quotation* travelled without it. `[R:names-travel-with-the-quote]`
#
# WHY THE TERM LIST IS NOT COMMITTED HERE, unlike in the source repository.
#
# There the list is a committed file of salted digests, and that is right:
# that repository is PRIVATE, so publishing `salt` alongside
# `sha256(salt || term)` costs nothing, and a committed list means a fresh
# clone can never arrive disarmed.
#
# This repository is PUBLIC, and one of the protected names normalises to four
# characters. 36^4 is about 1.7 million candidates — seconds of work against a
# published salt. Committing the list here would DISCLOSE the very thing it
# detects, which is the same self-reference failure as a detector that matches
# its own definition `[R:detector-excludes-own-definitions]`, pointing the
# other way. So the digests stay outside the repository and this script is the
# half that is safe to publish: it contains no names, only the matcher.
#
# WHERE THE LIST LIVES
#
#   $BANNED_TERMS_FILE                     if set
#   ~/.claude/usage/banned-terms.sha256    otherwise
#
# FORMAT (the source repository's, unchanged — see its scripts/banned-terms.sha256)
#   salt = <hex>              one line, prepended to every term before hashing
#   <len> <64 hex chars>      one per term: length of the normalised form,
#                             then sha256(salt_bytes || normalised_term)
#
# NORMALISED FORM: lowercase, ASCII alphanumerics only, separators dropped.
# Matching runs the same two passes as the source gate, because neither covers
# the other: JOINING every run of one to three consecutive tokens, so
# `ProductName`, `product-name` and `Product Name` all reduce to one candidate;
# and SLIDING every stored-length window inside each token, so a name fused to
# a neighbour (`ProductNameConfig`) is still found.
#
# EXIT CODES, and the middle one is the point:
#   0  scanned, clean
#   1  a banned name is present
#   2  DISARMED — no term list, or no perl to run the matcher
#
# 2 is a failure, never a pass. A check that reports the same green whether it
# is armed or absent is worse than none `[R:guarantee-needs-a-reader]`. The
# consequence is deliberate: an outside contributor has no list, so this script
# is NOT wired into `cargo test` or CI, where it would fail every fork for a
# reason that is none of their business. It is wired into `.githooks/pre-push`,
# which a maintainer installs once with
#
#     git config core.hooksPath .githooks
#
# so the person who actually has private names to protect is the person the
# gate is armed for. That makes this a CONFIGURATION-DEPENDENT control, and
# FEATURES.md says so rather than claiming a CI gate it does not have.
#
# A FINDING NEVER NAMES WHAT IT FOUND. Output is a path and a count. A check
# that printed the banned name into a terminal or a CI log would have moved the
# exposure rather than closed it `[R:report-the-hit-not-the-match]`.

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TERMS="${BANNED_TERMS_FILE:-$HOME/.claude/usage/banned-terms.sha256}"

if ! command -v perl >/dev/null 2>&1; then
    echo "[banned-names] DISARMED: perl not found; the matcher cannot run." >&2
    exit 2
fi

if [ ! -f "$TERMS" ]; then
    echo "[banned-names] DISARMED: no term list at $TERMS" >&2
    echo "[banned-names] Set BANNED_TERMS_FILE, or install the list, then re-run." >&2
    exit 2
fi

perl - "$TERMS" "$ROOT" <<'PERL'
use strict;
use warnings;
use Digest::SHA qw(sha256_hex);
use File::Find;

my ($terms, $root) = @ARGV;

# ── the list ────────────────────────────────────────────────────────────────
my ($salt, %digest, %lengths);
open my $fh, '<', $terms or die "cannot read $terms: $!\n";
while (my $line = <$fh>) {
    chomp $line;
    $line =~ s/^\s+|\s+$//g;
    next if $line =~ /^#/ || $line eq '';
    if ($line =~ /^salt\s*=\s*([0-9a-fA-F]+)$/) { $salt = pack("H*", lc $1); next; }
    if ($line =~ /^(\d+)\s+([0-9a-fA-F]{64})$/) {
        $digest{"$1:" . lc($2)} = 1;
        $lengths{$1} = 1;
    }
}
close $fh;

# An empty list is a disarmed gate, refused rather than reported clean.
unless (defined $salt && keys %digest) {
    print STDERR "[banned-names] DISARMED: $terms has no salt or no terms.\n";
    exit 2;
}
my @lengths = sort { $a <=> $b } keys %lengths;

sub is_banned {
    my $candidate = shift;
    return exists $digest{ length($candidate) . ":" . sha256_hex($salt . $candidate) };
}

# ── the matcher: join runs of up to three tokens, then slide inside each ─────
sub count_hits {
    my @tokens = (lc(shift) =~ /([a-z0-9]+)/g);
    my $hits = 0;
    for my $i (0 .. $#tokens) {
        for my $n (1 .. 3) {
            last if $i + $n - 1 > $#tokens;
            my $joined = join('', @tokens[ $i .. $i + $n - 1 ]);
            $hits++ if exists $lengths{ length $joined } && is_banned($joined);
        }
        my $token = $tokens[$i];
        for my $len (@lengths) {
            next if length($token) <= $len;
            for my $off (0 .. length($token) - $len) {
                $hits++ if is_banned(substr($token, $off, $len));
            }
        }
    }
    return $hits;
}

# ── the walk ────────────────────────────────────────────────────────────────
my %skip_dir = map { $_ => 1 } qw(.git target node_modules);
my %skip_ext = map { $_ => 1 } qw(png jpg jpeg ico gif wasm pdf zip exe lock);

my ($files, $total) = (0, 0);
find({
    no_chdir => 0,
    wanted   => sub {
        if (-d $_) {
            $File::Find::prune = 1 if $skip_dir{$_};
            # A DIRECTORY whose own name is the term: report the parent only.
            if ($_ ne '.' && count_hits($_)) {
                my $parent = $File::Find::name;
                $parent =~ s{/[^/]*$}{};
                print "  1  <directory name> under $parent/\n";
                $total++;
            }
            return;
        }
        return unless -f $_;
        my ($ext) = ($_ =~ /\.([^.]+)$/);
        return if defined $ext && $skip_ext{ lc $ext };

        my $rel = $File::Find::name;
        $rel =~ s/^\Q$root\E\/?//;

        # A FILENAME that is itself the term: printing the path would print the
        # name. Report the directory, never the last component.
        if (count_hits($_)) {
            my $dir = $rel;
            $dir =~ s{/[^/]*$}{};
            print "  1  <file name> in $dir/\n";
            $total++;
            return;
        }

        open my $in, '<:raw', $_ or return;
        local $/;
        my $body = <$in>;
        close $in;
        $files++;
        my $hits = count_hits($body);
        if ($hits) {
            printf "%3d  %s\n", $hits, $rel;
            $total += $hits;
        }
    },
}, $root);

if ($total) {
    print STDERR "[banned-names] FAILED: $total occurrence(s) across the tree.\n";
    print STDERR "[banned-names] Locations and counts are above; the term is deliberately not printed.\n";
    exit 1;
}
print "[banned-names] ok — $files file(s) scanned, no banned name present.\n";
exit 0;
PERL
