	.file	"main"
	.text
	.globl	main
	.p2align	4
	.type	main,@function
main:
	.cfi_startproc
	pushq	%rax
	.cfi_def_cfa_offset 16
	callq	test@PLT
	popq	%rcx
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end0:
	.size	main, .Lfunc_end0-main
	.cfi_endproc

	.globl	test
	.p2align	4
	.type	test,@function
test:
	.cfi_startproc
	movl	$42, %eax
	retq
.Lfunc_end1:
	.size	test, .Lfunc_end1-test
	.cfi_endproc

	.section	".note.GNU-stack","",@progbits
