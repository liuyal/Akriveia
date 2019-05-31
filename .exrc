

augroup Filetype_Latex
	autocmd!
	autocmd FileType tex,latex let &makeprg="pdflatex __Requirements_Specification.tex"
augroup END
