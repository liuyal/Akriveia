

augroup Filetype_Latex
	autocmd!
	autocmd FileType tex,latex let &makeprg="pdflatex _Requirements_Specification.tex"
augroup END
