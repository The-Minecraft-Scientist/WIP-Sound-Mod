package net.randomscientist.soundmod.mixins;

import net.minecraft.block.BlockState;
import net.minecraft.world.chunk.ChunkSection;
import net.minecraft.world.chunk.PalettedContainer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

@Mixin(ChunkSection.class)
public interface ChunkSectionAccessor {
	@Accessor
	short getNonEmptyBlockCount();
	@Accessor
	short getNonEmptyFluidCount();
	@Accessor
	PalettedContainer<BlockState> getBlockStateContainer();

}
