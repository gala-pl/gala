from setuptools import setup, Extension
import subprocess
import os

class TreeSitterGala(Extension):
    def __init__(self):
        super().__init__(
            'tree_sitter_gala._binding',
            sources=['bindings/python/tree_sitter_gala/__init__.py'],
        )

    def build_extension(self, ext):
        subprocess.check_call(['tree-sitter', 'generate'])

        import sysconfig
        cfg = sysconfig.get_config_vars()
        ext.include_dirs.append('src')

        if not os.path.exists('src/parser.c'):
            raise RuntimeError('Run "tree-sitter generate" first')

        from setuptools.command.build_ext import build_ext
        build_ext.build_extension(self, ext)


setup(
    name='tree-sitter-gala',
    version='0.1.0',
    description='Gala language parser for tree-sitter',
    packages=['tree_sitter_gala'],
    package_dir={'': 'bindings/python'},
    ext_modules=[TreeSitterBuild()],
)