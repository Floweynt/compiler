vim.api.nvim_create_autocmd('User', { 
    pattern = 'TSUpdate',
    callback = function()
        local p = require("nvim-treesitter.parsers");

        p.nodes = {
            install_info = {
                path = vim.fn.getcwd() .. "/tree-sitter-nodes",
                generate = true,
                generate_from_json = false,
                queries = "queries"
            },
            filetype = "nodes",
        };
        p.hir = {
            install_info = {
                path = vim.fn.getcwd() .. "/tree-sitter-hir",
                generate = true,
                generate_from_json = false,
                queries = "queries"
            },
            filetype = "hir",
        };
    end
})

require("nvim-treesitter.install").install("nodes")
require("nvim-treesitter.install").install("hir")

vim.filetype.add({
    extension = {
        nodes = 'nodes',
        hir = 'hir',
    }
})

