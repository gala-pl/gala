# typed: strict
# frozen_string_literal: true

# Gala: A hybrid quantum-classical programming language.
#
# To install from this tap:
#   brew tap gala-lang/gala
#   brew install gala
#
# With backends:
#   brew install gala --with-llvm --with-sim

class Gala < Formula
  desc "Hybrid quantum-classical programming language"
  homepage "https://gala-lang.org"
  license "Apache-2.0"
  head "https://github.com/gala-lang/gala.git", branch: "main"

  stable do
    version "0.1.0"
    url "https://github.com/gala-lang/gala/archive/refs/tags/v0.1.0.tar.gz"
    # Update this sha256 after each release:
    #   curl -sL https://github.com/gala-lang/gala/archive/refs/tags/v0.1.0.tar.gz | shasum -a 256
    sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  end

  livecheck do
    url :stable
    regex(/^v?(\d+(?:\.\d+)+)$/i)
  end

  # --- Dependencies ---

  depends_on "rust" => :build
  depends_on "cmake" => :build
  depends_on "pkg-config" => :build

  depends_on "llvm@17" => :optional
  depends_on "tree-sitter" => :optional

  # --- Options ---

  option "with-llvm", "Build with LLVM support for QIR emission and classical codegen"
  option "with-sim", "Build with quantum simulator backend (roqoqo/QuEST)"
  option "with-tree-sitter", "Build with tree-sitter grammar for syntax highlighting"

  # --- Build ---

  def install
    # Set LLVM prefix for inkwell/llvm-sys
    if build.with?("llvm")
      llvm = Formula["llvm@17"]
      ENV["LLVM_SYS_100_PREFIX"] = llvm.opt_prefix.to_s
      ohai "LLVM found at #{llvm.opt_prefix}"
    end

    # Assemble cargo feature flags
    features = %w[]
    features << "qir" if build.with?("llvm")
    features << "classical" if build.with?("llvm")
    features << "sim" if build.with?("sim")
    features << "tree-sitter" if build.with?("tree-sitter")

    # Install the main gala binary
    args = std_cargo_args(path: "crates/gala-cli")
    args << "--features" << features.join(",") unless features.empty?
    system "cargo", "install", *args

    # Install supporting tools
    system "cargo", "install", *std_cargo_args(path: "crates/gala-fmt")
    system "cargo", "install", *std_cargo_args(path: "crates/gala-lsp")
    system "cargo", "install", *std_cargo_args(path: "crates/gala-pkg")
  end

  # --- Post-install ---

  def post_install
    return unless build.with?("llvm")

    ohai "LLVM support enabled."
    puts "  Add LLVM to your PATH to use QIR features:"
    puts "  export PATH=\"$(brew --prefix llvm@17)/bin:$PATH\""
  end

  # --- Tests ---

  test do
    # Verify basic compilation
    (testpath/"test.gala").write "fn main() -> Int { return 42; }"
    output = shell_output("#{bin}/gala build test.gala 2>&1")
    assert_match "fn main", output

    # Verify the explain command works
    output = shell_output("#{bin}/gala explain E0401 2>&1")
    assert_match "Linearity", output

    # Verify version
    output = shell_output("#{bin}/gala --version 2>&1")
    assert_match "gala #{version}", output

    # Verify formatter
    output = shell_output("#{bin}/gala fmt test.gala 2>&1")
    assert_match "fn main", output
  end
end
