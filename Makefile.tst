.PHONY: ls tail cat_lock du 

_sandbox_guard:
	@echo hidden target

du:
	du -h
	
cat: ls ; tail -f /dev/null

echo:
	@echo test
ls:
	@ls -la

a.o:
	@ls -la

a:
	@ls -la

c     b:
	@ls -la

 tail:
	@tail -f /var/log/kern.log

cat_lock:
	@cat Cargo.lock

df free storage:
	@df -h

%::
	@true
