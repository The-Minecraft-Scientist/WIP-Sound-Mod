package net.randomscientist.soundmod.util;

public class BitOffset {
	private int bits;
	BitOffset(int bits) {
		this.bits = bits;
	}
	public int asNumBits() {
		return this.bits;
	}
	public int asNumBytes() {
		return this.bits >> 3;
	}
	public int asNumShorts() {
		return this.bits >> 4;
	}
	public int asNumInts() {
		return this.bits >> 5;
	}
	public int asNumLongs() {
		return this.bits >> 6;
	}
	public void setBits(int numBits) {
		this.bits = numBits;
	}

	public void incrementBits(int numBits) {
		this.bits += numBits;
	}
	public void incrementBytes(int numBytes) {
		this.bits += numBytes << 3;
	}
	public void incrementShorts(int numShorts) {
		this.bits += numShorts << 4;
	}
	public void incrementInts(int numInts) {
		this.bits += numInts << 5;
	}
	public void incrementLongs(int numLongs) {
		this.bits += numLongs << 6;
	}
}
