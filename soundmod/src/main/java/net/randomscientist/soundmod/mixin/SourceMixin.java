package net.randomscientist.soundmod.mixin;

import net.minecraft.client.sound.Source;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Source.class)
public class SourceMixin {
	private long uuid;
	private static int counter;
	/**
	 * @author
	 * The-Minecraft-Scientist
	 * @reason
	 * rewrite sound backend
	 */
	@Overwrite(aliases="create")
	static Source create() {
		counter++;
		return new Source((int) 0);
	}

	@Inject(at = @At("TAIL"), method = "<init>")
	public void Source(int pointer, CallbackInfo ci) {
	}

}
