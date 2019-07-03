

augroup Filetype_Latex
	autocmd!
	autocmd FileType tex,latex let &makeprg="pdflatex _Design_Specification.tex"
augroup END
