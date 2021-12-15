# $1 = package

export LFS=/home/edvin/mnt/lfs
source $LFS/sources/$1.pkgbuild
wget $source -P $LFS/var/cache/packman/pkg/
tarfile=$(echo $source | egrep -o "$pkgname-.*$")
tarfile_md5=$(md5sum $LFS/var/cache/packman/pkg/$tarfile | egrep -o "^[[:alnum:]]*")
if [ $md5sums != $tarfile_md5 ]; then
	rm $LFS/var/cache/packman/pkg/$tarfile
	echo "md5sum failed"
	exit 1
else
	exit 0
fi
