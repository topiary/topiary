{
  languages = {
    bash = {
      extensions | default = ["sh", "bash"],
      grammar.source | default = {
        git = {
          git = "https://github.com/tree-sitter/tree-sitter-bash.git",
          rev = "d1a1a3fe7189fdab5bd29a54d1df4a5873db5cb1",
          nixHash = "sha256-XiiEI7/6b2pCZatO8Z8fBgooKD8Z+SFQJNdR/sGGkgE=",
        }
      },
    },

    css = {
      extensions | default = ["css"],
      grammar.source | default = {
        git = {
          git = "https://github.com/tree-sitter/tree-sitter-css.git",
          rev = "6e327db434fec0ee90f006697782e43ec855adf5",
          nixHash = "sha256-en379DlqzzvQNvKgE8CtiA00j7phUyipttqbnETGHKc=",
        },
      },
    },

    json = {
      extensions | default = [
        "json",
        "avsc",
        "geojson",
        "gltf",
        "har",
        "ice",
        "JSON-tmLanguage",
        "jsonl",
        "mcmeta",
        "tfstate",
        "tfstate.backup",
        "topojson",
        "webapp",
        "webmanifest"
      ],
      grammar.source | default = {
        git = {
          git = "https://github.com/tree-sitter/tree-sitter-json.git",
          rev = "v0.24.8",
          nixHash = "sha256-DNZC2cTy1C8OaMOpEHM6NoRtOIbLaBf0CLXXWCKODlw=",
        },
      },
    },
    markdown = {
      extensions = ["md"],
      grammar.source.git = {
        git = "https://github.com/tree-sitter-grammars/tree-sitter-markdown.git",
        rev = "413285231ce8fa8b11e7074bbe265b48aa7277f9",
        nixHash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
      },
    },

    nickel = {
      extensions | default = ["ncl"],
      grammar.source | default = {
        git = {
          git = "https://github.com/nickel-lang/tree-sitter-nickel",
          rev = "488ee4e6af15e10dd4be527777c9ba18a817d407",
          nixHash = "sha256-CMlf80y1te30HwjT9ykHeg6xvQc/lcCCHDMVGs7oVXQ=",
        },
      },
    },

    ocaml = {
      extensions | default = ["ml"],
      grammar.source | default = {
        git = {
          git = "https://github.com/tree-sitter/tree-sitter-ocaml.git",
          rev = "a45fda5fe73cda92f2593d16340b3f6bd46674b8",
          subdir = "grammars/ocaml",
          nixHash = "sha256-u8R3JvjaOrW6kCX1hNTGMl86HnKAoopSMvHr8Sj0i04=",
        },
      },
    },

    ocaml_interface = {
      extensions | default = ["mli"],
      grammar.source | default = {
        git = {
          git = "https://github.com/tree-sitter/tree-sitter-ocaml.git",
          rev = "a45fda5fe73cda92f2593d16340b3f6bd46674b8",
          subdir = "grammars/interface",
          nixHash = "sha256-u8R3JvjaOrW6kCX1hNTGMl86HnKAoopSMvHr8Sj0i04=",
        },
      },
    },

    ocamllex = {
      extensions | default = ["mll"],
      grammar.source | default = {
        git = {
          git = "https://github.com/314eter/tree-sitter-ocamllex.git",
          rev = "5da5bb7508ac9fd3317561670ef18c126a0fe2aa",
          nixHash = "sha256-qfmIfcZ3zktYzuNNYP7Z6u6c7XoKsKD86MRMxe/qkpY=",
        },
      },
    },

    openscad = {
      extensions | default = ["scad"],
      grammar.source | default = {
        git = {
          git = "https://github.com/openscad/tree-sitter-openscad.git",
          rev = "acc196e969a169cadd8b7f8d9f81ff2d30e3e253",
          nixHash = "sha256-x6fU1yPhYfoXemjswk+yRHW+c5V6nJgesK5tunYE7MI=",
        },
      },
    },

    rust = {
      extensions | default = ["rs"],
      indent | default = "    ", # 4 spaces
      grammar.source | default = {
        git = {
          git = "https://github.com/tree-sitter/tree-sitter-rust.git",
          rev = "e0e8b6de6e4aa354749c794f5f36a906dcccda74",
          nixHash = "sha256-egTxBuliboYbl+5N6Jdt960EMLByVmLqSmQLps3rEok=",
        },
      },
    },

    sdml = {
      extensions | default = ["sdm", "sdml"],
      grammar.source | default = {
        git = {
          git = "https://github.com/sdm-lang/tree-sitter-sdml",
          rev = "056fc1d0b8624aa4e58967c67bb129ebdfa6d563",
          nixHash = "sha256-UrlhkcfdBFN/o+NhSPDqfCS0/OpLByVA/5NPl+JPD58=",
        },
      },
    },

    toml = {
      extensions | default = ["toml"],
      grammar.source | default = {
        git = {
          git = "https://github.com/tree-sitter/tree-sitter-toml.git",
          rev = "342d9be207c2dba869b9967124c679b5e6fd0ebe",
          nixHash = "sha256-5nLNBxFeOGE+gzbwpcrTVnuL1jLUA0ZLBVw2QrOLsDQ=",
        },
      },
    },

    tree_sitter_query = {
      extensions | default = ["scm"],
      grammar = {
        source | default = {
          git = {
            git = "https://github.com/nvim-treesitter/tree-sitter-query",
            rev = "a0ccc351e5e868ec1f8135e97aa3b53c663cf2df",
            nixHash = "sha256-H2QLsjl3/Kh0ojCf2Df38tb9KrM2InphEmtGd0J6+hM=",
          },
        },
        symbol = "tree_sitter_query",
      },
    },

    wit = {
      extensions | default = ["wit"],
      indent | default = "    ", # 4 spaces
      grammar.source | default = {
        git = {
          git = "https://github.com/bytecodealliance/tree-sitter-wit",
          rev = "v1.2.0",
          nixHash = "sha256-scye60ETUak1mXJXC+UY5sqbuqAcjxCsm4+AVJHhGws=",
        },
      },
    },
  }
}
